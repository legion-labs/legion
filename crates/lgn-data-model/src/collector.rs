use crate::{utils::ReflectionError, FieldDescriptor, TypeDefinition, TypeReflection};
/// Info about a Property
pub struct ItemInfo<'a> {
    /// Pointer to the raw_data
    pub base: *const (),
    /// Type of the Property
    pub type_def: TypeDefinition,
    /// Field Descriptor
    pub field_descriptor: Option<&'a FieldDescriptor>,
    /// Name Suffix  (such as array '[0]')
    pub suffix: Option<&'a str>,
    /// Depth of traveral
    pub depth: usize,
}

/// Trait to collect reflection data
pub trait PropertyCollector {
    /// Type created by the collector
    type Item;

    /// Callback to create a new Item for a field
    fn new_item(info: &ItemInfo<'_>) -> anyhow::Result<Self::Item>
    where
        Self: Sized;

    /// Callback to add child item to a parent
    fn add_child(parent: &mut Self::Item, child: Self::Item)
    where
        Self: Sized;
}

/// Collect all the properties of a `TypeReflection`
pub fn collect_properties<T>(base: &dyn TypeReflection) -> anyhow::Result<T::Item>
where
    T: PropertyCollector,
{
    let item_info = ItemInfo {
        base: (base as *const dyn TypeReflection).cast::<()>(),
        type_def: base.get_type(),
        suffix: None,
        depth: 0,
        field_descriptor: None,
    };
    internal_collect_properties::<T>(&item_info)
}

fn internal_collect_properties<T>(item_info: &ItemInfo<'_>) -> anyhow::Result<T::Item>
where
    T: PropertyCollector,
{
    let result = match item_info.type_def {
        TypeDefinition::None => {
            let resource_type = item_info
                .field_descriptor
                .map_or(item_info.suffix.unwrap_or_default(), |f| {
                    f.field_name.as_str()
                });

            return Err(ReflectionError::InvalidateTypeDescriptor(resource_type.into()).into());
        }
        TypeDefinition::BoxDyn(box_dyn_descriptor) => {
            // For BoxDyn, pipe directly to the inner type
            let sub_base = unsafe { (box_dyn_descriptor.get_inner)(item_info.base) };
            let sub_type = unsafe { (box_dyn_descriptor.get_inner_type)(item_info.base) };
            internal_collect_properties::<T>(&ItemInfo {
                base: sub_base,
                type_def: sub_type,
                suffix: item_info.suffix,
                depth: item_info.depth,
                field_descriptor: item_info.field_descriptor,
            })?
        }

        TypeDefinition::Array(array_descriptor) => {
            let mut array_parent = T::new_item(item_info)?;
            for index in 0..unsafe { (array_descriptor.len)(item_info.base) } {
                let child = internal_collect_properties::<T>(&ItemInfo {
                    base: unsafe { (array_descriptor.get)(item_info.base, index) }?,
                    type_def: array_descriptor.inner_type,
                    suffix: Some(format!("[{}]", index).as_str()),
                    depth: item_info.depth + 1,
                    field_descriptor: None,
                })?;
                T::add_child(&mut array_parent, child);
            }
            array_parent
        }

        TypeDefinition::Primitive(_primitive_descriptor) => T::new_item(item_info)?,

        TypeDefinition::Option(option_descriptor) => {
            let mut option_parent = T::new_item(item_info)?;
            if let Some(value_base) = unsafe { (option_descriptor.get_inner)(item_info.base) } {
                let child = internal_collect_properties::<T>(&ItemInfo {
                    base: value_base,
                    type_def: option_descriptor.inner_type,
                    suffix: None,
                    depth: item_info.depth + 1,
                    field_descriptor: item_info.field_descriptor,
                })?;
                T::add_child(&mut option_parent, child);
            }
            option_parent
        }
        TypeDefinition::Struct(struct_descriptor) => {
            let mut struct_parent = T::new_item(item_info)?;
            struct_descriptor
                .fields
                .iter()
                .try_for_each(|field| -> anyhow::Result<()> {
                    let field_base =
                        unsafe { item_info.base.cast::<u8>().add(field.offset).cast::<()>() };
                    let child = internal_collect_properties::<T>(&ItemInfo {
                        base: field_base,
                        type_def: field.field_type,
                        suffix: None,
                        depth: item_info.depth + 1,
                        field_descriptor: Some(field),
                    })?;
                    T::add_child(&mut struct_parent, child);
                    Ok(())
                })?;
            struct_parent
        }
    };
    Ok(result)
}
