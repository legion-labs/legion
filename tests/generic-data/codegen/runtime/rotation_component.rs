use lgn_math::prelude::*;
#[derive(serde :: Serialize, serde :: Deserialize, PartialEq)]
pub struct RotationComponent {
    pub rotation_speed: Vec3,
}
impl RotationComponent {
    #[allow(dead_code)]
    const SIGNATURE_HASH: u64 = 18233231219042076698u64;
    #[allow(dead_code)]
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
impl lgn_data_model::TypeReflection for RotationComponent {
    fn get_type(&self) -> lgn_data_model::TypeDefinition {
        Self::get_type_def()
    }
    #[allow(unused_mut)]
    #[allow(clippy::let_and_return)]
    fn get_type_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_struct_descriptor!(
            RotationComponent,
            vec![lgn_data_model::FieldDescriptor {
                field_name: "rotation_speed".into(),
                offset: memoffset::offset_of!(RotationComponent, rotation_speed),
                field_type: <Vec3 as lgn_data_model::TypeReflection>::get_type_def(),
                attributes: {
                    let mut attr = std::collections::HashMap::new();
                    attr
                }
            },]
        );
        lgn_data_model::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_option_descriptor!(RotationComponent);
        lgn_data_model::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_array_descriptor!(RotationComponent);
        lgn_data_model::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { # [allow (clippy :: needless_update)] static ref __ROTATIONCOMPONENT_DEFAULT : RotationComponent = RotationComponent :: default () ; }
#[typetag::serde(name = "Runtime_RotationComponent")]
impl lgn_data_runtime::Component for RotationComponent {
    fn eq(&self, other: &dyn lgn_data_runtime::Component) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| std::cmp::PartialEq::eq(self, other))
    }
}
