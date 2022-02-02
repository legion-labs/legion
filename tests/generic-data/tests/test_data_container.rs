#![allow(unsafe_code)]

use std::collections::HashMap;
use std::io::Cursor;

use generic_data::offline::{TestComponent, TestEntity, TestEntityProcessor, TestSubType2};
use lgn_data_model::collector::{collect_properties, ItemInfo, PropertyCollector};
use lgn_data_model::json_utils::{get_property_as_json_string, set_property_from_json_string};
use lgn_data_model::TypeReflection;
use lgn_data_runtime::AssetLoader;
use lgn_math::prelude::*;

#[test]
fn test_default_implementation() {
    let entity = TestEntity::default();
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

#[test]
fn test_write_field_by_name() {
    let mut entity = TestEntity::default();
    entity
        .test_sub_type
        .test_components
        .push(Box::new(TestComponent { test_i32: 1337 }));

    entity.test_option_set = Some(TestSubType2 {
        test_vec: (1.0, 2.0, 3.0).into(),
    });

    entity.test_option_primitive_set = Some((1.0, 2.0, 3.0).into());

    set_property_from_json_string(&mut entity, "test_string", "\"New Value\"").unwrap();
    assert_eq!(entity.test_string, "New Value");

    set_property_from_json_string(&mut entity, "test_position", "[1.1, -2.2, 3.3]").unwrap();
    assert_eq!(entity.test_position, Vec3::new(1.1, -2.2, 3.3));

    set_property_from_json_string(&mut entity, "test_rotation", "[0,1,0,0]").unwrap();
    assert_eq!(entity.test_rotation, Quat::from_xyzw(0.0, 1.0, 0.0, 0.0));

    set_property_from_json_string(&mut entity, "test_bool", " true").unwrap();
    assert!(entity.test_bool);

    set_property_from_json_string(&mut entity, "test_float32", " 1.23 ").unwrap();
    assert!((entity.test_float32 - 1.23).abs() < f32::EPSILON);

    set_property_from_json_string(&mut entity, "test_float64", "  2.45 ").unwrap();
    assert!((entity.test_float64 - 2.45).abs() < f64::EPSILON);

    set_property_from_json_string(&mut entity, "test_int", " -10").unwrap();
    assert_eq!(entity.test_int, -10);

    set_property_from_json_string(&mut entity, "test_blob", "[4,5,6,7]").unwrap();
    assert_eq!(entity.test_blob, vec![4, 5, 6, 7]);

    set_property_from_json_string(&mut entity, "test_sub_type.test_string", "\"NewValue\"")
        .unwrap();
    assert_eq!(entity.test_sub_type.test_string, "NewValue");

    // Test Parsing sub properties
    set_property_from_json_string(
        &mut entity,
        "test_sub_type.test_components[0].test_i32",
        "1338",
    )
    .unwrap();

    let value = get_property_as_json_string(&entity, "test_option_set.test_vec").unwrap();
    assert_eq!(value, "[1.0,2.0,3.0]");

    let value = get_property_as_json_string(&entity, "test_option_primitive_set").unwrap();
    println!("value: {}", value);

    // Test trying to get an empty option (should fail)
    let result = get_property_as_json_string(&entity, "test_option_none.test_vec");
    assert!(!result.is_ok());

    let serde_json = serde_json::to_string(&entity).unwrap();
    let dynamic_serde_json = get_property_as_json_string(&entity, "").unwrap();
    assert_eq!(serde_json, dynamic_serde_json);
}

#[test]
fn test_editor_descriptors() {
    // Test Static type info (codegen)
    u32::get_type_def();
    f32::get_type_def();
    Option::<u32>::get_type_def();
    Vec::<u32>::get_type_def();

    // Test Dynamic type info
    let entity = TestEntity::default();

    entity.get_type();
}

#[test]
fn test_collector() {
    struct PropertyBag {
        name: String,
        ptype: String,
        sub_properties: Vec<PropertyBag>,
        attributes: Option<HashMap<String, String>>,
    }

    impl PropertyCollector for PropertyBag {
        type Item = Self;
        fn new_item(item_info: &ItemInfo<'_>) -> anyhow::Result<Self::Item> {
            Ok(Self::Item {
                name: item_info
                    .field_descriptor
                    .map_or(String::new(), |field| field.field_name.clone())
                    + item_info.suffix.unwrap_or_default(),
                ptype: item_info.type_def.get_type_name().into(),
                sub_properties: Vec::new(),
                attributes: item_info
                    .field_descriptor
                    .and_then(|field| field.attributes.clone()),
            })
        }
        fn add_child(parent: &mut Self::Item, child: Self::Item) {
            let sub_properties = &mut parent.sub_properties;

            // If there's a 'Group' attribute, find or create a PropertyBag for the Group within the parent
            if let Some(Some(group_name)) =
                child.attributes.as_ref().map(|attrs| attrs.get("group"))
            {
                // Search for the Group within the Parent SubProperties

                let group_bag = if let Some(group_bag) = sub_properties
                    .iter_mut()
                    .find(|bag| bag.ptype == "_group_" && bag.name == *group_name)
                {
                    group_bag
                } else {
                    // Create a new group bag if not found
                    sub_properties.push(Self::Item {
                        name: group_name.into(),
                        ptype: "_group_".into(),
                        sub_properties: Vec::new(),
                        attributes: None,
                    });
                    sub_properties.last_mut().unwrap()
                };

                // Add child to group
                group_bag.sub_properties.push(child);
            } else {
                sub_properties.push(child);
            }
        }
    }

    // Test Dynamic type info
    let entity = TestEntity::default();
    let output = collect_properties::<PropertyBag>(&entity).unwrap();
    assert_eq!(output.ptype, "TestEntity");
    assert_eq!(output.sub_properties.len(), 13);
    assert_eq!(output.sub_properties[0].name, "test_string");
    assert_eq!(output.sub_properties[0].ptype, "String");
    assert_eq!(output.sub_properties[1].name, "GroupTest1");
    assert_eq!(output.sub_properties[1].ptype, "_group_");
}
