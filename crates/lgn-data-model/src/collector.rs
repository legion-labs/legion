use crate::{utils::ReflectionError, FieldDescriptor, TypeDefinition, TypeReflection};
/// Info about a Property
pub struct ItemInfo<'a> {
    /// Pointer to the raw_data
    pub base: *const (),
    /// Type of the Property
    pub type_def: TypeDefinition,
    /// Field Descriptor
    pub field_descriptor: Option<&'a FieldDescriptor>,
    /// Name Suffix  (such as array '`[`0`]`')
    pub suffix: Option<&'a str>,
    /// Depth of traveral
    pub depth: usize,
}

/// Trait to collect reflection data
pub trait PropertyCollector {
    /// Type created by the collector
    type Item;

    /// Callback to create a new Item for a field
    fn new_item(info: &ItemInfo<'_>) -> Result<Self::Item, ReflectionError>
    where
        Self: Sized;

    /// Callback to add child item to a parent
    fn add_child(parent: &mut Self::Item, child: Self::Item)
    where
        Self: Sized;
}

/// Collect all the properties of a `TypeReflection`
pub fn collect_properties<T>(base: &dyn TypeReflection) -> Result<T::Item, ReflectionError>
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
    item_info.collect::<T>()
}

impl ItemInfo<'_> {
    /// Collect all the properties of a `ItemInfo` (subfield)
    pub fn collect<T>(&self) -> Result<T::Item, ReflectionError>
    where
        T: PropertyCollector,
    {
        let result = match self.type_def {
            TypeDefinition::None => {
                let resource_type = self
                    .field_descriptor
                    .map_or(self.suffix.unwrap_or_default(), |f| f.field_name.as_str());

                return Err(ReflectionError::InvalidTypeDescriptor(resource_type.into()));
            }
            TypeDefinition::BoxDyn(box_dyn_descriptor) => {
                // For BoxDyn, pipe directly to the inner type
                let sub_base = (box_dyn_descriptor.get_inner)(self.base);
                let sub_type = (box_dyn_descriptor.get_inner_type)(self.base);
                let obj = ItemInfo {
                    base: sub_base,
                    type_def: sub_type,
                    suffix: self.suffix,
                    depth: self.depth,
                    field_descriptor: self.field_descriptor,
                };
                obj.collect::<T>()?
            }

            TypeDefinition::Array(array_descriptor) => {
                let mut array_parent = T::new_item(self)?;
                for index in 0..(array_descriptor.len)(self.base) {
                    let base = (array_descriptor.get)(self.base, index)?;
                    let array_index =
                        if let TypeDefinition::BoxDyn(box_dyn) = array_descriptor.inner_type {
                            format!("[{}]", { (box_dyn.get_inner_type)(base).get_type_name() })
                        } else {
                            format!("[{}]", index)
                        };

                    let child = ItemInfo {
                        base,
                        type_def: array_descriptor.inner_type,
                        suffix: Some(array_index.as_str()),
                        depth: self.depth + 1,
                        field_descriptor: None,
                    };
                    T::add_child(&mut array_parent, child.collect::<T>()?);
                }
                array_parent
            }

            TypeDefinition::Enum(_enum_descriptor) => T::new_item(self)?,
            TypeDefinition::Primitive(_primitive_descriptor) => T::new_item(self)?,

            TypeDefinition::Option(option_descriptor) => {
                let mut option_parent = T::new_item(self)?;
                if let Some(value_base) = unsafe { (option_descriptor.get_inner)(self.base) } {
                    let child = ItemInfo {
                        base: value_base,
                        type_def: option_descriptor.inner_type,
                        suffix: None,
                        depth: self.depth + 1,
                        field_descriptor: self.field_descriptor,
                    };
                    T::add_child(&mut option_parent, child.collect::<T>()?);
                }
                option_parent
            }
            TypeDefinition::Struct(struct_descriptor) => {
                let mut struct_parent = T::new_item(self)?;
                struct_descriptor.fields.iter().try_for_each(
                    |field| -> Result<(), ReflectionError> {
                        let field_base =
                            unsafe { self.base.cast::<u8>().add(field.offset).cast::<()>() };
                        let child = ItemInfo {
                            base: field_base,
                            type_def: field.field_type,
                            suffix: None,
                            depth: self.depth + 1,
                            field_descriptor: Some(field),
                        };
                        T::add_child(&mut struct_parent, child.collect::<T>()?);
                        Ok(())
                    },
                )?;
                struct_parent
            }
        };
        Ok(result)
    }
}
