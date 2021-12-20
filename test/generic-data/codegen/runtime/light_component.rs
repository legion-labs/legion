use lgn_graphics_data::Color;
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct LightComponent {
    pub light_color: Color,
}
impl LightComponent {
    #[allow(dead_code)]
    const SIGNATURE_HASH: u64 = 8640207041695829339u64;
    #[allow(dead_code)]
    pub fn get_default_instance() -> &'static Self {
        &__LIGHTCOMPONENT_DEFAULT
    }
}
#[allow(clippy::derivable_impls)]
impl Default for LightComponent {
    fn default() -> Self {
        Self {
            light_color: (255, 255, 255, 255).into(),
        }
    }
}
impl lgn_data_model::TypeReflection for LightComponent {
    fn get_type(&self) -> lgn_data_model::TypeDefinition {
        Self::get_type_def()
    }
    fn get_type_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_struct_descriptor!(
            LightComponent,
            vec![lgn_data_model::FieldDescriptor {
                field_name: "light_color".into(),
                offset: memoffset::offset_of!(LightComponent, light_color),
                field_type: <Color as lgn_data_model::TypeReflection>::get_type_def(),
                group: "".into()
            },]
        );
        lgn_data_model::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_option_descriptor!(LightComponent);
        lgn_data_model::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_array_descriptor!(LightComponent);
        lgn_data_model::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { # [allow (clippy :: needless_update)] static ref __LIGHTCOMPONENT_DEFAULT : LightComponent = LightComponent :: default () ; }
#[typetag::serde(name = "Runtime_LightComponent")]
impl lgn_data_runtime::Component for LightComponent {}
