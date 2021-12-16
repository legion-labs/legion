#![allow(dead_code)]
#![allow(clippy::needless_update)]

use lgn_graphics_data::Color;
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct StaticMeshComponent {
    pub mesh_id: usize,
    pub color: Color,
}
impl StaticMeshComponent {
    const SIGNATURE_HASH: u64 = 17359756617277276696u64;
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
impl lgn_data_reflection::TypeReflection for StaticMeshComponent {
    fn get_type(&self) -> lgn_data_reflection::TypeDefinition {
        Self::get_type_def()
    }
    fn get_type_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_struct_descriptor!(
            StaticMeshComponent,
            vec![
                lgn_data_reflection::FieldDescriptor {
                    field_name: "mesh_id".into(),
                    offset: memoffset::offset_of!(StaticMeshComponent, mesh_id),
                    field_type: <usize as lgn_data_reflection::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_reflection::FieldDescriptor {
                    field_name: "color".into(),
                    offset: memoffset::offset_of!(StaticMeshComponent, color),
                    field_type: <Color as lgn_data_reflection::TypeReflection>::get_type_def(),
                    group: "".into()
                },
            ]
        );
        lgn_data_reflection::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_option_descriptor!(StaticMeshComponent);
        lgn_data_reflection::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_array_descriptor!(StaticMeshComponent);
        lgn_data_reflection::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { static ref __STATICMESHCOMPONENT_DEFAULT : StaticMeshComponent = StaticMeshComponent { .. StaticMeshComponent :: default () } ; }
#[typetag::serde(name = "StaticMeshComponent")]
impl lgn_data_runtime::Component for StaticMeshComponent {}
