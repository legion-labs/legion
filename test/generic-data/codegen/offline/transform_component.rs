#![allow(dead_code)]
#![allow(clippy::needless_update)]

use lgn_math::prelude::*;
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct TransformComponent {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
impl TransformComponent {
    const SIGNATURE_HASH: u64 = 13214906858233497312u64;
    pub fn get_default_instance() -> &'static Self {
        &__TRANSFORMCOMPONENT_DEFAULT
    }
}
#[allow(clippy::derivable_impls)]
impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0, 0.0).into(),
            rotation: Quat::IDENTITY,
            scale: (1.0, 1.0, 1.0).into(),
        }
    }
}
impl lgn_data_reflection::TypeReflection for TransformComponent {
    fn get_type(&self) -> lgn_data_reflection::TypeDefinition {
        Self::get_type_def()
    }
    fn get_type_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_struct_descriptor!(
            TransformComponent,
            vec![
                lgn_data_reflection::FieldDescriptor {
                    field_name: "position".into(),
                    offset: memoffset::offset_of!(TransformComponent, position),
                    field_type: <Vec3 as lgn_data_reflection::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_reflection::FieldDescriptor {
                    field_name: "rotation".into(),
                    offset: memoffset::offset_of!(TransformComponent, rotation),
                    field_type: <Quat as lgn_data_reflection::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_reflection::FieldDescriptor {
                    field_name: "scale".into(),
                    offset: memoffset::offset_of!(TransformComponent, scale),
                    field_type: <Vec3 as lgn_data_reflection::TypeReflection>::get_type_def(),
                    group: "".into()
                },
            ]
        );
        lgn_data_reflection::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_option_descriptor!(TransformComponent);
        lgn_data_reflection::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_array_descriptor!(TransformComponent);
        lgn_data_reflection::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { static ref __TRANSFORMCOMPONENT_DEFAULT : TransformComponent = TransformComponent { .. TransformComponent :: default () } ; }
#[typetag::serde(name = "TransformComponent")]
impl lgn_data_runtime::Component for TransformComponent {}
