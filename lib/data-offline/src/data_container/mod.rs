//! `DataContainer`
use crate::ResourcePathId;
use legion_math::prelude::*;

/// Property Descriptor
pub struct PropertyDescriptor {
    /// Name of the Property
    pub name: &'static str,
    /// Type of the Property
    pub type_name: &'static str,
    /// Default value of the property
    pub default_value: Vec<u8>,
    /// Current value of the property
    pub value: Vec<u8>,
    /// Group of the property
    pub group: String,
}

/// Proc-Macro Trait for `DataContainer`
pub trait OfflineDataContainer {
    /// Create a `DataContainer` from a Json
    fn read_from_json(&mut self, json_data: &str) -> std::io::Result<()>;

    /// Write a `DataContainer` as a JSON stream to file/writer
    fn write_to_json(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()>;

    /// Compile a Offline `DataContainer` to it's Runtime binary representation
    fn compile_runtime(&self) -> Result<Vec<u8>, &'static str>;

    /// Write a field by name using reflection
    fn write_field_by_name(
        &mut self,
        field_name: &str,
        field_value: &str,
    ) -> Result<(), &'static str>;

    /// Return the Editor Property Descriptor
    fn get_editor_properties(&self) -> Result<Vec<PropertyDescriptor>, &'static str>;

    /// Signature of `DataContainer` used for compilation dependencies
    const SIGNATURE_HASH: u64;
}

/// Trait for implement parsing of `DataContainer` properties
pub trait ParseFromStr {
    /// Parse from string
    fn parse_from_str(&mut self, field_value: &str) -> Result<(), &'static str>;
}

/// Macro to implement default implementation for type that support std::str::FromStr
#[macro_export]
macro_rules! implement_parse_from_str {
    ($($name:ident),+) => {
        $(
            impl ParseFromStr for $name {
                fn parse_from_str(&mut self, field_value: &str) -> Result<(), &'static str> {
                    *self = field_value.trim().parse().map_err(|_err| "error parsing")?;
                    Ok(())
                }
            }
        )+
    }
}

implement_parse_from_str!(bool, u32, i32, i64, u64, f32, f64);

impl ParseFromStr for String {
    fn parse_from_str(&mut self, field_value: &str) -> Result<(), &'static str> {
        *self = field_value.into();
        Ok(())
    }
}

impl ParseFromStr for Vec3 {
    fn parse_from_str(&mut self, field_value: &str) -> Result<(), &'static str> {
        let words: Vec<&str> = field_value.split_terminator(',').collect();
        if words.len() != 3 {
            return Err("error parsing Vec3, expected 3 values");
        }
        let x: f32 = words[0]
            .trim()
            .parse()
            .map_err(|_err| "error parsing Vec3, invalid float")?;
        let y: f32 = words[1]
            .trim()
            .parse()
            .map_err(|_err| "error parsing Vec3, invalid float")?;
        let z: f32 = words[2]
            .trim()
            .parse()
            .map_err(|_err| "error parsing Vec3, invalid float")?;
        *self = (x, y, z).into();
        Ok(())
    }
}

impl ParseFromStr for Quat {
    fn parse_from_str(&mut self, field_value: &str) -> Result<(), &'static str> {
        let words: Vec<&str> = field_value.split_terminator(',').collect();
        if words.len() != 4 {
            return Err("error parsing Quat, expected 4 values");
        }
        let x: f32 = words[0]
            .trim()
            .parse()
            .map_err(|_err| "error parsing Quat, invalid float")?;
        let y: f32 = words[1]
            .trim()
            .parse()
            .map_err(|_err| "error parsing Quat, invalid float")?;
        let z: f32 = words[2]
            .trim()
            .parse()
            .map_err(|_err| "error parsing Quat, invalid float")?;
        let w: f32 = words[3]
            .trim()
            .parse()
            .map_err(|_err| "error parsing Quat, invalid float")?;
        *self = Self::from_xyzw(x, y, z, w);
        Ok(())
    }
}

impl ParseFromStr for Vec<u8> {
    fn parse_from_str(&mut self, field_value: &str) -> Result<(), &'static str> {
        *self = field_value.as_bytes().to_vec();
        Ok(())
    }
}

impl ParseFromStr for Option<ResourcePathId> {
    fn parse_from_str(&mut self, field_value: &str) -> Result<(), &'static str> {
        if field_value.is_empty() {
            *self = None;
        } else {
            let res_id: ResourcePathId = field_value
                .parse()
                .map_err(|_err| "invalid resourcePathId")?;
            *self = Some(res_id);
        }
        Ok(())
    }
}
