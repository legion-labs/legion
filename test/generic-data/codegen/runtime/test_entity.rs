use lgn_data_runtime::Component;
use lgn_graphics_data::Color;
use lgn_math::prelude::*;
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct TestEntity {
    pub test_string: String,
    pub test_color: Color,
    pub test_position: Vec3,
    pub test_rotation: Quat,
    pub test_bool: bool,
    pub test_float32: f32,
    pub test_int: i32,
    pub test_blob: Vec<u8>,
    pub test_sub_type: TestSubType1,
    pub test_option_set: Option<TestSubType2>,
    pub test_option_none: Option<TestSubType2>,
}
impl TestEntity {
    #[allow(dead_code)]
    const SIGNATURE_HASH: u64 = 5274493235039250438u64;
    #[allow(dead_code)]
    pub fn get_default_instance() -> &'static Self {
        &__TESTENTITY_DEFAULT
    }
}
#[allow(clippy::derivable_impls)]
impl Default for TestEntity {
    fn default() -> Self {
        Self {
            test_string: "string literal".into(),
            test_color: (255, 0, 0, 255).into(),
            test_position: (0.0, 0.0, 0.0).into(),
            test_rotation: Quat::IDENTITY,
            test_bool: false,
            test_float32: 32.32f32,
            test_int: 123,
            test_blob: [0, 1, 2, 3].into(),
            test_sub_type: TestSubType1::default(),
            test_option_set: None,
            test_option_none: None,
        }
    }
}
impl lgn_data_model::TypeReflection for TestEntity {
    fn get_type(&self) -> lgn_data_model::TypeDefinition {
        Self::get_type_def()
    }
    fn get_type_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_struct_descriptor!(
            TestEntity,
            vec![
                lgn_data_model::FieldDescriptor {
                    field_name: "test_string".into(),
                    offset: memoffset::offset_of!(TestEntity, test_string),
                    field_type: <String as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "test_color".into(),
                    offset: memoffset::offset_of!(TestEntity, test_color),
                    field_type: <Color as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "test_position".into(),
                    offset: memoffset::offset_of!(TestEntity, test_position),
                    field_type: <Vec3 as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "test_rotation".into(),
                    offset: memoffset::offset_of!(TestEntity, test_rotation),
                    field_type: <Quat as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "test_bool".into(),
                    offset: memoffset::offset_of!(TestEntity, test_bool),
                    field_type: <bool as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "test_float32".into(),
                    offset: memoffset::offset_of!(TestEntity, test_float32),
                    field_type: <f32 as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "test_int".into(),
                    offset: memoffset::offset_of!(TestEntity, test_int),
                    field_type: <i32 as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "test_blob".into(),
                    offset: memoffset::offset_of!(TestEntity, test_blob),
                    field_type: <Vec<u8> as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "test_sub_type".into(),
                    offset: memoffset::offset_of!(TestEntity, test_sub_type),
                    field_type: <TestSubType1 as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "test_option_set".into(),
                    offset: memoffset::offset_of!(TestEntity, test_option_set),
                    field_type:
                        <Option<TestSubType2> as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "test_option_none".into(),
                    offset: memoffset::offset_of!(TestEntity, test_option_none),
                    field_type:
                        <Option<TestSubType2> as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
            ]
        );
        lgn_data_model::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_option_descriptor!(TestEntity);
        lgn_data_model::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_array_descriptor!(TestEntity);
        lgn_data_model::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { # [allow (clippy :: needless_update)] static ref __TESTENTITY_DEFAULT : TestEntity = TestEntity :: default () ; }
use lgn_data_runtime::{Asset, AssetLoader, Resource};
use std::{any::Any, io};
impl Resource for TestEntity {
    const TYPENAME: &'static str = "runtime_testentity";
}
impl Asset for TestEntity {
    type Loader = TestEntityLoader;
}
#[derive(Default)]
pub struct TestEntityLoader {}
impl AssetLoader for TestEntityLoader {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let output: TestEntity = bincode::deserialize_from(reader)
            .map_err(|_err| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse"))?;
        Ok(Box::new(output))
    }
    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct TestComponent {
    pub test_i32: i32,
}
impl TestComponent {
    #[allow(dead_code)]
    const SIGNATURE_HASH: u64 = 16512715240131344153u64;
    #[allow(dead_code)]
    pub fn get_default_instance() -> &'static Self {
        &__TESTCOMPONENT_DEFAULT
    }
}
#[allow(clippy::derivable_impls)]
impl Default for TestComponent {
    fn default() -> Self {
        Self {
            test_i32: i32::default(),
        }
    }
}
impl lgn_data_model::TypeReflection for TestComponent {
    fn get_type(&self) -> lgn_data_model::TypeDefinition {
        Self::get_type_def()
    }
    fn get_type_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_struct_descriptor!(
            TestComponent,
            vec![lgn_data_model::FieldDescriptor {
                field_name: "test_i32".into(),
                offset: memoffset::offset_of!(TestComponent, test_i32),
                field_type: <i32 as lgn_data_model::TypeReflection>::get_type_def(),
                group: "".into()
            },]
        );
        lgn_data_model::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_option_descriptor!(TestComponent);
        lgn_data_model::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_array_descriptor!(TestComponent);
        lgn_data_model::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { # [allow (clippy :: needless_update)] static ref __TESTCOMPONENT_DEFAULT : TestComponent = TestComponent :: default () ; }
#[typetag::serde(name = "Runtime_TestComponent")]
impl lgn_data_runtime::Component for TestComponent {}
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct TestSubType1 {
    pub test_components: Vec<Box<dyn Component>>,
    pub test_string: String,
    pub test_sub_type: TestSubType2,
}
impl TestSubType1 {
    #[allow(dead_code)]
    const SIGNATURE_HASH: u64 = 10652788437003811010u64;
    #[allow(dead_code)]
    pub fn get_default_instance() -> &'static Self {
        &__TESTSUBTYPE1_DEFAULT
    }
}
#[allow(clippy::derivable_impls)]
impl Default for TestSubType1 {
    fn default() -> Self {
        Self {
            test_components: Vec::new(),
            test_string: String::default(),
            test_sub_type: TestSubType2::default(),
        }
    }
}
impl lgn_data_model::TypeReflection for TestSubType1 {
    fn get_type(&self) -> lgn_data_model::TypeDefinition {
        Self::get_type_def()
    }
    fn get_type_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_struct_descriptor!(
            TestSubType1,
            vec![
                lgn_data_model::FieldDescriptor {
                    field_name: "test_components".into(),
                    offset: memoffset::offset_of!(TestSubType1, test_components),
                    field_type:
                        <Vec<Box<dyn Component>> as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "test_string".into(),
                    offset: memoffset::offset_of!(TestSubType1, test_string),
                    field_type: <String as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "test_sub_type".into(),
                    offset: memoffset::offset_of!(TestSubType1, test_sub_type),
                    field_type: <TestSubType2 as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
            ]
        );
        lgn_data_model::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_option_descriptor!(TestSubType1);
        lgn_data_model::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_array_descriptor!(TestSubType1);
        lgn_data_model::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { # [allow (clippy :: needless_update)] static ref __TESTSUBTYPE1_DEFAULT : TestSubType1 = TestSubType1 :: default () ; }
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct TestSubType2 {
    pub test_vec: Vec3,
}
impl TestSubType2 {
    #[allow(dead_code)]
    const SIGNATURE_HASH: u64 = 16122499844266623450u64;
    #[allow(dead_code)]
    pub fn get_default_instance() -> &'static Self {
        &__TESTSUBTYPE2_DEFAULT
    }
}
#[allow(clippy::derivable_impls)]
impl Default for TestSubType2 {
    fn default() -> Self {
        Self {
            test_vec: Vec3::default(),
        }
    }
}
impl lgn_data_model::TypeReflection for TestSubType2 {
    fn get_type(&self) -> lgn_data_model::TypeDefinition {
        Self::get_type_def()
    }
    fn get_type_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_struct_descriptor!(
            TestSubType2,
            vec![lgn_data_model::FieldDescriptor {
                field_name: "test_vec".into(),
                offset: memoffset::offset_of!(TestSubType2, test_vec),
                field_type: <Vec3 as lgn_data_model::TypeReflection>::get_type_def(),
                group: "".into()
            },]
        );
        lgn_data_model::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_option_descriptor!(TestSubType2);
        lgn_data_model::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_array_descriptor!(TestSubType2);
        lgn_data_model::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { # [allow (clippy :: needless_update)] static ref __TESTSUBTYPE2_DEFAULT : TestSubType2 = TestSubType2 :: default () ; }
