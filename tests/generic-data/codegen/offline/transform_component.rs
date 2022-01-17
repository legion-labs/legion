use lgn_math::prelude::*;
#[derive(serde :: Serialize, serde :: Deserialize, PartialEq)]
pub struct TransformComponent {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
impl TransformComponent {
    #[allow(dead_code)]
    const SIGNATURE_HASH: u64 = 1257914175696485568u64;
    #[allow(dead_code)]
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
impl lgn_data_model::TypeReflection for TransformComponent {
    fn get_type(&self) -> lgn_data_model::TypeDefinition {
        Self::get_type_def()
    }
    #[allow(unused_mut)]
    #[allow(clippy::let_and_return)]
    #[allow(clippy::too_many_lines)]
    fn get_type_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_struct_descriptor!(
            TransformComponent,
            vec![
                lgn_data_model::FieldDescriptor {
                    field_name: "position".into(),
                    offset: memoffset::offset_of!(TransformComponent, position),
                    field_type: <Vec3 as lgn_data_model::TypeReflection>::get_type_def(),
                    attributes: {
                        let mut attr = std::collections::HashMap::new();
                        attr
                    }
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "rotation".into(),
                    offset: memoffset::offset_of!(TransformComponent, rotation),
                    field_type: <Quat as lgn_data_model::TypeReflection>::get_type_def(),
                    attributes: {
                        let mut attr = std::collections::HashMap::new();
                        attr
                    }
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "scale".into(),
                    offset: memoffset::offset_of!(TransformComponent, scale),
                    field_type: <Vec3 as lgn_data_model::TypeReflection>::get_type_def(),
                    attributes: {
                        let mut attr = std::collections::HashMap::new();
                        attr
                    }
                },
            ]
        );
        lgn_data_model::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_option_descriptor!(TransformComponent);
        lgn_data_model::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_array_descriptor!(TransformComponent);
        lgn_data_model::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { # [allow (clippy :: needless_update)] static ref __TRANSFORMCOMPONENT_DEFAULT : TransformComponent = TransformComponent :: default () ; }
#[typetag::serde(name = "TransformComponent")]
impl lgn_data_runtime::Component for TransformComponent {
    fn eq(&self, other: &dyn lgn_data_runtime::Component) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| std::cmp::PartialEq::eq(self, other))
    }
}
