use crate::type_reflection::TypeReflection;
use crate::utils::{deserialize_property_by_name, serialize_property_by_name};
use crate::ReflectionError;

/// Read a Property as a bincode value
pub fn get_property_as_bincode(
    object: &dyn TypeReflection,
    path: &str,
) -> Result<Vec<u8>, ReflectionError> {
    let mut buffer = Vec::new();

    let mut json = bincode::Serializer::new(&mut buffer, bincode::DefaultOptions::new());
    let mut serializer = <dyn erased_serde::Serializer>::erase(&mut json);
    serialize_property_by_name(object, path, &mut serializer)?;
    Ok(buffer)
}

/// Write a Property as a bincode value
pub fn set_property_from_bincode(
    object: &mut dyn TypeReflection,
    path: &str,
    value: &[u8],
) -> Result<(), ReflectionError> {
    let mut json = bincode::de::Deserializer::with_reader(value, bincode::DefaultOptions::new());
    let mut deserializer = <dyn erased_serde::Deserializer<'_>>::erase(&mut json);
    deserialize_property_by_name(object, path, &mut deserializer)
}
