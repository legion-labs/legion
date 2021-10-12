//! `DataContainer`
use legion_math::prelude::*;

/// Proc-Macro Trait for `DataContainer`
pub trait OfflineDataContainer {
    /// Create a `DataContainer` from a Json
    fn read_from_json(&mut self, json_data: &str) -> std::io::Result<()>;

    /// Write a `DataContainer` as a JSON stream to file/writer
    fn write_to_json(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()>;

    /// Compile a Offline `DataContainer` to it's Runtime binary representation
    fn compile_runtime(&self) -> Result<Vec<u8>, String>;

    /// Write a field by name using reflection
    fn write_field_by_name(
        &mut self,
        field_name: &str,
        field_value: &str,
    ) -> Result<(), &'static str>;

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
                    *self = field_value.parse().map_err(|_err| "error parsing")?;
                    Ok(())
                }
            }
        )+
    }
}

implement_parse_from_str!(String, bool, u32, i32, i64, u64, f32, f64);

impl ParseFromStr for Vec3 {
    fn parse_from_str(&mut self, field_value: &str) -> Result<(), &'static str> {
        let words: Vec<&str> = field_value.split_whitespace().collect();
        let x: f32 = words[0].parse().map_err(|_err| "error parsing")?;
        let y: f32 = words[1].parse().map_err(|_err| "error parsing")?;
        let z: f32 = words[2].parse().map_err(|_err| "error parsing")?;
        *self = (x, y, z).into();
        Ok(())
    }
}

impl ParseFromStr for Quat {
    fn parse_from_str(&mut self, field_value: &str) -> Result<(), &'static str> {
        let words: Vec<&str> = field_value.split_whitespace().collect();
        let x: f32 = words[0].parse().map_err(|_err| "error parsing")?;
        let y: f32 = words[1].parse().map_err(|_err| "error parsing")?;
        let z: f32 = words[2].parse().map_err(|_err| "error parsing")?;
        let w: f32 = words[3].parse().map_err(|_err| "error parsing")?;
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
