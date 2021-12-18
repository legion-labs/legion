use crate::{utils::ReflectionError, TypeDefinition, TypeReflection};
use itertools::Itertools;

/// Trait to collect reflection data
pub trait PropertyCollector {
    /// Type created by the collector
    type Item;

    /// Callback to create a new Item for a field
    fn new_item(
        base: *const (),
        type_def: TypeDefinition,
        name: &str,
    ) -> anyhow::Result<Self::Item>
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
    internal_collect_properties::<T>(
        (base as *const dyn TypeReflection).cast::<()>(),
        base.get_type(),
        base.get_type().get_type_name(),
    )
}

fn internal_collect_properties<T>(
    base: *const (),
    type_def: TypeDefinition,
    property_name: &str,
) -> anyhow::Result<T::Item>
where
    T: PropertyCollector,
{
    let mut parent = T::new_item(base, type_def, property_name)?;

    match type_def {
        TypeDefinition::None => {
            return Err(ReflectionError::InvalidateTypeDescriptor(property_name.into()).into());
        }
        TypeDefinition::BoxDyn(box_dyn_descriptor) => {
            let sub_base = unsafe { (box_dyn_descriptor.get_inner)(base) };
            let sub_type = unsafe { (box_dyn_descriptor.get_inner_type)(base) };
            let child = internal_collect_properties::<T>(sub_base, sub_type, property_name)?;
            T::add_child(&mut parent, child);
        }

        TypeDefinition::Array(array_descriptor) => {
            for index in 0..unsafe { (array_descriptor.len)(base) } {
                let element_base = unsafe { (array_descriptor.get)(base, index) }?;
                let child = internal_collect_properties::<T>(
                    element_base,
                    array_descriptor.inner_type,
                    format!("[{}]", index).as_str(),
                )?;
                T::add_child(&mut parent, child);
            }
        }

        TypeDefinition::Primitive(_primitive_descriptor) => {}

        #[allow(clippy::option_if_let_else)]
        TypeDefinition::Option(option_descriptor) => {
            if let Some(value_base) = unsafe { (option_descriptor.get_inner)(base) } {
                let child =
                    internal_collect_properties::<T>(value_base, option_descriptor.inner_type, "")?;
                T::add_child(&mut parent, child);
            }
        }
        TypeDefinition::Struct(struct_descriptor) => {
            struct_descriptor
                .fields
                .iter()
                .group_by(|f| f.group.as_str())
                .into_iter()
                .try_for_each(|(group_name, fields)| -> anyhow::Result<()> {
                    let mut group: Option<T::Item> = None;

                    if !group_name.is_empty() {
                        group = Some(T::new_item(
                            std::ptr::null(),
                            TypeDefinition::None,
                            group_name,
                        )?);
                    }
                    for field in fields {
                        let field_base =
                            unsafe { base.cast::<u8>().add(field.offset).cast::<()>() };
                        let child = internal_collect_properties::<T>(
                            field_base,
                            field.field_type,
                            field.field_name.as_str(),
                        )?;

                        if let Some(group) = &mut group {
                            T::add_child(group, child);
                        } else {
                            T::add_child(&mut parent, child);
                        }
                    }

                    if let Some(group) = group {
                        T::add_child(&mut parent, group);
                    }

                    Ok(())
                })?;
        }
    }
    Ok(parent)
}
