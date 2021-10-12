use legion_data_offline::data_container::{OfflineDataContainer, ParseFromStr, PropertyDescriptor};
pub use legion_data_offline_macros::DataContainer;
use legion_math::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{BufReader, Read};
use std::{fs::File, io::Write};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum EnumTest {
    Value0,
    Value1,
    Value2,
}

impl ParseFromStr for EnumTest {
    fn parse_from_str(&mut self, field_value: &str) -> Result<(), &'static str> {
        *self = match field_value.trim() {
            "Value0" => EnumTest::Value0,
            "Value1" => EnumTest::Value1,
            "Value2" => EnumTest::Value2,
            _ => return Err("invalid enum"),
        };
        Ok(())
    }
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

#[test]
fn test_write_field_by_name() {
    let mut entity = TestEntity {
        ..Default::default()
    };

    entity
        .write_field_by_name("test_string", "New Value")
        .unwrap();
    assert_eq!(entity.test_string, "New Value");

    entity
        .write_field_by_name("test_position", "1.1, -2.2, 3.3")
        .unwrap();
    assert_eq!(entity.test_position, Vec3::new(1.1, -2.2, 3.3));

    entity
        .write_field_by_name("test_rotation", "0,1,0,0")
        .unwrap();
    assert_eq!(entity.test_rotation, Quat::from_xyzw(0.0, 1.0, 0.0, 0.0));

    entity.write_field_by_name("test_bool", " true").unwrap();
    assert!(entity.test_bool);

    entity
        .write_field_by_name("test_float32", " 1.23 ")
        .unwrap();
    assert!((entity.test_float32 - 1.23).abs() < f32::EPSILON);

    entity
        .write_field_by_name("test_float64", "  2.45 ")
        .unwrap();
    assert!((entity.test_float64 - 2.45).abs() < f64::EPSILON);

    entity.write_field_by_name("test_enum", "Value1").unwrap();
    assert_eq!(entity.test_enum, EnumTest::Value1);

    entity.write_field_by_name("test_int", " -10").unwrap();
    assert_eq!(entity.test_int, -10);

    entity
        .write_field_by_name("test_blob", "\x04\x05\x06\x07")
        .unwrap();
    assert_eq!(entity.test_blob, vec![4, 5, 6, 7]);
}

#[test]
fn test_editor_descriptors() {
    let entity = TestEntity {
        ..Default::default()
    };

    let _descriptors = entity.get_editor_properties().unwrap();
}
