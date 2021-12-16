#![allow(dead_code)]
#![allow(clippy::needless_update)]

use lgn_math::prelude::*;
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct RotationComponent {
    pub rotation_speed: Vec3,
}
impl RotationComponent {
    const SIGNATURE_HASH: u64 = 96410917651568727u64;
    pub fn get_default_instance() -> &'static Self {
        &__ROTATIONCOMPONENT_DEFAULT
    }
}
#[allow(clippy::derivable_impls)]
impl Default for RotationComponent {
    fn default() -> Self {
        Self {
            rotation_speed: (0.0, 0.0, 0.0).into(),
        }
    }
}
impl lgn_data_reflection::TypeReflection for RotationComponent {
    fn get_type(&self) -> lgn_data_reflection::TypeDefinition {
        Self::get_type_def()
    }
    fn get_type_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_struct_descriptor!(
            RotationComponent,
            vec![lgn_data_reflection::FieldDescriptor {
                field_name: "rotation_speed".into(),
                offset: memoffset::offset_of!(RotationComponent, rotation_speed),
                field_type: <Vec3 as lgn_data_reflection::TypeReflection>::get_type_def(),
                group: "".into()
            },]
        );
        lgn_data_reflection::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_option_descriptor!(RotationComponent);
        lgn_data_reflection::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_array_descriptor!(RotationComponent);
        lgn_data_reflection::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { static ref __ROTATIONCOMPONENT_DEFAULT : RotationComponent = RotationComponent { .. RotationComponent :: default () } ; }
#[typetag::serde(name = "RotationComponent")]
impl lgn_data_runtime::Component for RotationComponent {}
