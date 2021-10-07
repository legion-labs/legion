//! `DataContainer`

/// Proc-Macro Trait for `DataContainer`
pub trait OfflineDataContainer {
    /// Create a `DataContainer` from a Json
    fn read_from_json(&mut self, json_data: &str) -> std::io::Result<()>;

    /// Write a `DataContainer` as a JSON stream to file/writer
    fn write_to_json(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()>;

    /// Compile a Offline `DataContainer` to it's Runtime binary representation
    fn compile_runtime(&self) -> Result<Vec<u8>, String>;

    /// Signature of `DataContainer` used for compilation dependencies
    const SIGNATURE_HASH: u64;
}

#[cfg(test)]
mod tests {

    use crate::data_container::OfflineDataContainer;
    pub use legion_data_offline_macros::DataContainer;
    use legion_math::prelude::*;
    use serde::{Deserialize, Serialize};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    use std::io::{BufReader, Read};
    use std::{fs::File, io::Write};

    #[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
    pub enum EnumTest {
        Value0,
        Value1,
        Value2,
    }

    fn func_hash_test(salt: u32, val: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write_u32(salt);
        hasher.write(val.as_bytes());
        hasher.finish()
    }

    #[derive(DataContainer)]
    pub struct TestEntity {
        // Default with string literal
        #[legion(default = "string literal", readonly, category = "Name")]
        test_string: String,

        // Default with Tuple()
        #[legion(default=(0.0,0.0,0.0), hidden)]
        pub test_position: Vec3,

        // Default with Constant value
        #[legion(default= Quat::IDENTITY, tooltip = "Rotation Tooltip")]
        pub test_rotation: Quat,

        // Default initialized from func call
        #[legion(default = func_hash_test(0x1234,"test"), transient)]
        pub test_transient: u64,

        // Default with bool constant
        #[legion(default = false)]
        test_bool: bool,

        // Default with Float constant
        #[legion(default = 32.32f32)]
        test_float32: f32,

        #[legion(default = 64.64f64, offline)]
        test_float64: f64,

        // Default with Enum
        #[legion(default = EnumTest::Value0, readonly)]
        pub test_enum: EnumTest,

        // Default with Integer constant
        #[legion(default = 123)]
        test_int: i32,

        // Default with Array
        #[legion(default=[0,1,2,3])]
        test_blob: Vec<u8>,
    }

    #[test]
<<<<<<< HEAD
    fn test_default_implementation() {
        let entity = TestEntity {
            ..Default::default()
        };

        assert_eq!(entity.test_string.as_str(), "string literal");
        assert_eq!(entity.test_position, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(entity.test_rotation, Quat::IDENTITY);
        assert_eq!(entity.test_transient, func_hash_test(0x1234, "test"));
        assert!(!entity.test_bool);
        assert!((entity.test_float32 - 32.32f32).abs() < f32::EPSILON);
        assert!((entity.test_float64 - 64.64f64).abs() < f64::EPSILON);
        assert_eq!(entity.test_enum, EnumTest::Value0);
        assert_eq!(entity.test_int, 123);
        assert_eq!(entity.test_blob, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_json_serialization() {
        let mut entity = TestEntity {
            ..Default::default()
        };

        let json_data = r#"
            {
                "_class" : "TestEntity",
                "test_string" : "Value read from json",
                "test_position" : [2,2,2],
                "test_rotation" : [0,0,0,2],
                "test_bool" : true,
                "test_float32" : 3232.32,
                "test_float64" : 6464.64,
                "test_int" : 1000,
                "test_blob" : [3,2,1,0]
            }"#;

        entity.read_from_json(json_data).unwrap();
        assert_eq!(entity.test_string.as_str(), "Value read from json");
        assert_eq!(entity.test_position, Vec3::new(2.0, 2.0, 2.0));
        assert_eq!(entity.test_rotation, Quat::from_xyzw(0.0, 0.0, 0.0, 2.0));
        assert!(entity.test_bool);
        assert!((entity.test_float32 - 3232.32f32).abs() < f32::EPSILON);
        assert!((entity.test_float64 - 6464.64f64).abs() < f64::EPSILON);
        assert_eq!(entity.test_int, 1000);
        assert_eq!(entity.test_blob, vec![3, 2, 1, 0]);
    }

    #[test]
    fn test_compile_data_container() {
        let entity = TestEntity {
            ..Default::default()
=======
    fn test_entity_serialization() {
        let json_data = r#"
        {
            "_class" : "Entity",
            "_base" : "ResourcePathId",
            "name": "EntityA",
            "test_bool" : true,
            "test_int" : 345345,
            "test_float" : 345.678,
            "test_vec3" : [2,2,2]
        }"#;

        let _default_instance = Entity {
            ..Entity::default()
        };

        let _test = RuntimeEntity {
            ..RuntimeEntity::default()
>>>>>>> f7209cc0 (new lints in data-offline, data-compiler)
        };
        let compiled_asset = entity.compile_runtime().unwrap();

        let root = tempfile::tempdir().unwrap();
        let temp_output = root.path().join("testEntity.bin");

        let mut file = File::create(&temp_output).unwrap();
        file.write_all(&compiled_asset).unwrap();
        file.flush().unwrap();

        if let Ok(file) = File::open(&temp_output) {
            let mut buf_reader = BufReader::new(file);

            let mut buffer: Vec<u8> = Vec::new();
            if buf_reader.read_to_end(&mut buffer).is_ok() {
                let runtime_asset: RuntimeTestEntity<'_> = bincode::deserialize(&buffer).unwrap();

                assert_eq!(runtime_asset.test_string, "string literal");
                assert_eq!(runtime_asset.test_position, Vec3::new(0.0, 0.0, 0.0));
                assert_eq!(runtime_asset.test_rotation, Quat::IDENTITY);
                assert!(!runtime_asset.test_bool);
                assert!((runtime_asset.test_float32 - 32.32f32).abs() < f32::EPSILON);
                assert_eq!(runtime_asset.test_enum, EnumTest::Value0);
                assert_eq!(runtime_asset.test_int, 123);
                assert_eq!(runtime_asset.test_blob, vec![0, 1, 2, 3]);
            }
        }
    }
}
