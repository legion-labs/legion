// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
#![allow(unsafe_code)]

use std::io::Cursor;

use generic_data_offline::{TestComponent, TestEntity, TestEntityProcessor, TestSubType2};
use lgn_data_reflection::collector::{collect_properties, PropertyCollector};
use lgn_data_reflection::json_utils::{get_property_as_json_string, set_property_from_json_string};
use lgn_data_reflection::{TypeDefinition, TypeReflection};
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
    enum PropertyBagValue {
        JsonString(String),
        SubProperties(Vec<PropertyBag>),
    }
    struct PropertyBag {
        name: String,
        ptype: String,
        value: PropertyBagValue,
    }

    impl PropertyCollector for PropertyBag {
        type Item = Self;
        fn new_item(
            base: *const (),
            type_def: TypeDefinition,
            name: &str,
        ) -> anyhow::Result<Self::Item> {
            if let TypeDefinition::Primitive(primitive_descriptor) = type_def {
                let mut output = Vec::new();
                let mut json = serde_json::Serializer::new(&mut output);
                let mut serializer = <dyn erased_serde::Serializer>::erase(&mut json);
                unsafe {
                    (primitive_descriptor.base_descriptor.dynamic_serialize)(
                        base,
                        &mut serializer,
                    )?;
                }

                Ok(Self {
                    name: name.into(),
                    ptype: primitive_descriptor.base_descriptor.type_name.clone(),
                    value: PropertyBagValue::JsonString(String::from_utf8(output)?),
                })
            } else {
                Ok(Self {
                    name: name.into(),
                    ptype: type_def.get_type_name().into(),
                    value: PropertyBagValue::SubProperties(Vec::new()),
                })
            }
        }
        fn add_child(parent: &mut Self::Item, child: Self::Item) {
            if let PropertyBagValue::SubProperties(prop) = &mut parent.value {
                prop.push(child);
            }
        }
    }

    // Test Dynamic type info
    let entity = TestEntity::default();
    let output = collect_properties::<PropertyBag>(&entity).unwrap();
    assert_eq!(output.name, "TestEntity");
    assert_eq!(output.ptype, "TestEntity");
    if let PropertyBagValue::SubProperties(sub) = output.value {
        assert_eq!(sub.len(), 12);
        assert_eq!(sub[0].name, "test_string");
        assert_eq!(sub[0].ptype, "String");
    } else {
        panic!("TestEntity doesn't have subproperty");
    }
}
