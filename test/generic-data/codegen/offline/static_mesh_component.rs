use lgn_graphics_data::Color;
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct StaticMeshComponent {
    pub mesh_id: usize,
    pub color: Color,
}
impl StaticMeshComponent {
    #[allow(dead_code)]
    const SIGNATURE_HASH: u64 = 13871379992308584785u64;
    #[allow(dead_code)]
    pub fn get_default_instance() -> &'static Self {
        &__STATICMESHCOMPONENT_DEFAULT
    }
}
#[allow(clippy::derivable_impls)]
impl Default for StaticMeshComponent {
    fn default() -> Self {
        Self {
            mesh_id: 0,
            color: (255, 0, 0, 255).into(),
        }
    }
}
impl lgn_data_model::TypeReflection for StaticMeshComponent {
    fn get_type(&self) -> lgn_data_model::TypeDefinition {
        Self::get_type_def()
    }
    #[allow(unused_mut)]
    #[allow(clippy::let_and_return)]
    #[allow(clippy::too_many_lines)]
    fn get_type_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_struct_descriptor!(
            StaticMeshComponent,
            vec![
                lgn_data_model::FieldDescriptor {
                    field_name: "mesh_id".into(),
                    offset: memoffset::offset_of!(StaticMeshComponent, mesh_id),
                    field_type: <usize as lgn_data_model::TypeReflection>::get_type_def(),
                    attributes: {
                        let mut attr = std::collections::HashMap::new();
                        attr
                    }
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "color".into(),
                    offset: memoffset::offset_of!(StaticMeshComponent, color),
                    field_type: <Color as lgn_data_model::TypeReflection>::get_type_def(),
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
        lgn_data_model::implement_option_descriptor!(StaticMeshComponent);
        lgn_data_model::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_array_descriptor!(StaticMeshComponent);
        lgn_data_model::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { # [allow (clippy :: needless_update)] static ref __STATICMESHCOMPONENT_DEFAULT : StaticMeshComponent = StaticMeshComponent :: default () ; }
#[typetag::serde(name = "StaticMeshComponent")]
impl lgn_data_runtime::Component for StaticMeshComponent {}
