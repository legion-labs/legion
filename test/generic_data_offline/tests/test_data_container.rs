use legion_data_runtime::AssetLoader;
use legion_math::prelude::*;
use std::io::Cursor;

use generic_data_offline::{TestEntity, TestEntityProcessor};
use legion_data_offline::resource::ResourceReflection;

#[test]
fn test_default_implementation() {
    let entity = TestEntity {
        ..Default::default()
    };

    assert_eq!(entity.test_string.as_str(), "string literal");
    assert_eq!(entity.test_position, Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(entity.test_rotation, Quat::IDENTITY);
    assert!(!entity.test_bool);
    assert!((entity.test_float32 - 32.32f32).abs() < f32::EPSILON);
    assert!((entity.test_float64 - 64.64f64).abs() < f64::EPSILON);
    assert_eq!(entity.test_int, 123);
    assert_eq!(entity.test_blob, vec![0, 1, 2, 3]);
}

#[test]
fn test_json_serialization() {
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

    let mut file = Cursor::new(json_data);

    let mut processor = TestEntityProcessor {};
    let entity = processor
        .load(&mut file)
        .unwrap()
        .downcast::<TestEntity>()
        .unwrap();

    assert_eq!(entity.test_string.as_str(), "Value read from json");
    assert_eq!(entity.test_position, Vec3::new(2.0, 2.0, 2.0));
    assert_eq!(entity.test_rotation, Quat::from_xyzw(0.0, 0.0, 0.0, 2.0));
    assert!(entity.test_bool);
    assert!((entity.test_float32 - 3232.32f32).abs() < f32::EPSILON);
    assert!((entity.test_float64 - 6464.64f64).abs() < f64::EPSILON);
    assert_eq!(entity.test_int, 1000);
    assert_eq!(entity.test_blob, vec![3, 2, 1, 0]);
}

/*
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
            let runtime_asset: runtime_data::TestEntity<'_> =
                bincode::deserialize(&buffer).unwrap();

            assert_eq!(runtime_asset.test_string, "string literal");
            assert_eq!(runtime_asset.test_position, Vec3::new(0.0, 0.0, 0.0));
            assert_eq!(runtime_asset.test_rotation, Quat::IDENTITY);
            assert!(!runtime_asset.test_bool);
            assert!((runtime_asset.test_float32 - 32.32f32).abs() < f32::EPSILON);
            assert_eq!(runtime_asset.test_int, 123);
            assert_eq!(runtime_asset.test_blob, vec![0, 1, 2, 3]);
        }
    }
}*/

#[test]
fn test_write_field_by_name() {
    let mut entity = TestEntity {
        ..Default::default()
    };

    entity
        .write_property("test_string", "\"New Value\"")
        .unwrap();
    assert_eq!(entity.test_string, "New Value");

    entity
        .write_property("test_position", "[1.1, -2.2, 3.3]")
        .unwrap();
    assert_eq!(entity.test_position, Vec3::new(1.1, -2.2, 3.3));

    entity.write_property("test_rotation", "[0,1,0,0]").unwrap();
    assert_eq!(entity.test_rotation, Quat::from_xyzw(0.0, 1.0, 0.0, 0.0));

    entity.write_property("test_bool", " true").unwrap();
    assert!(entity.test_bool);

    entity.write_property("test_float32", " 1.23 ").unwrap();
    assert!((entity.test_float32 - 1.23).abs() < f32::EPSILON);

    entity.write_property("test_float64", "  2.45 ").unwrap();
    assert!((entity.test_float64 - 2.45).abs() < f64::EPSILON);

    entity.write_property("test_int", " -10").unwrap();
    assert_eq!(entity.test_int, -10);

    entity.write_property("test_blob", "[4,5,6,7]").unwrap();
    assert_eq!(entity.test_blob, vec![4, 5, 6, 7]);
}

#[test]
fn test_editor_descriptors() {
    let entity = TestEntity {
        ..Default::default()
    };

    entity.get_property_descriptors().unwrap();
}
