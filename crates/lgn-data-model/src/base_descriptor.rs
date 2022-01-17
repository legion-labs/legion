/// Define the reflection of a Primitive type
pub struct BaseDescriptor {
    /// Type of the Property
    pub type_name: String,
    /// Size of the property
    pub size: usize,
    /// Function to dynamically serialize the Primitive from a raw ptr
    pub dynamic_serialize: unsafe fn(
        property: *const (),
        format: &mut dyn erased_serde::Serializer,
    ) -> anyhow::Result<()>,
    /// Function to dynamically deserialize the Primitive from a raw ptr
    pub dynamic_deserialize: unsafe fn(
        property: *mut (),
        format: &mut dyn erased_serde::Deserializer<'_>,
    ) -> anyhow::Result<()>,
}

/// Create the instantiate a `BaseDescriptor` with basic serde accessor
#[macro_export]
macro_rules! create_base_descriptor {
    ($type_id:ty, $type_name:expr) => {
        $crate::BaseDescriptor {
            type_name: $type_name,
            size: std::mem::size_of::<$type_id>(),
            dynamic_serialize:
                |property: *const (), serializer: &mut dyn::erased_serde::Serializer| unsafe {
                    ::erased_serde::serialize(&(*property.cast::<$type_id>()), serializer)?;
                    Ok(())
                },
            dynamic_deserialize:
                |property: *mut (), deserializer: &mut dyn::erased_serde::Deserializer<'_>| unsafe {
                    *(property.cast::<$type_id>()) = ::erased_serde::deserialize(deserializer)?;
                    Ok(())
                },
        }
    };
}
