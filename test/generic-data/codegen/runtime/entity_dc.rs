use lgn_data_runtime::Component;
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct EntityDc {
    pub name: String,
    pub components: Vec<Box<dyn Component>>,
}
impl EntityDc {
    #[allow(dead_code)]
    const SIGNATURE_HASH: u64 = 8800754876911975167u64;
    #[allow(dead_code)]
    pub fn get_default_instance() -> &'static Self {
        &__ENTITYDC_DEFAULT
    }
}
#[allow(clippy::derivable_impls)]
impl Default for EntityDc {
    fn default() -> Self {
        Self {
            name: "unnamed".into(),
            components: Vec::new(),
        }
    }
}
impl lgn_data_model::TypeReflection for EntityDc {
    fn get_type(&self) -> lgn_data_model::TypeDefinition {
        Self::get_type_def()
    }
    #[allow(unused_mut)]
    #[allow(clippy::let_and_return)]
    #[allow(clippy::too_many_lines)]
    fn get_type_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_struct_descriptor!(
            EntityDc,
            vec![
                lgn_data_model::FieldDescriptor {
                    field_name: "name".into(),
                    offset: memoffset::offset_of!(EntityDc, name),
                    field_type: <String as lgn_data_model::TypeReflection>::get_type_def(),
                    attributes: {
                        let mut attr = std::collections::HashMap::new();
                        attr
                    }
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "components".into(),
                    offset: memoffset::offset_of!(EntityDc, components),
                    field_type:
                        <Vec<Box<dyn Component>> as lgn_data_model::TypeReflection>::get_type_def(),
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
        lgn_data_model::implement_option_descriptor!(EntityDc);
        lgn_data_model::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_array_descriptor!(EntityDc);
        lgn_data_model::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { # [allow (clippy :: needless_update)] static ref __ENTITYDC_DEFAULT : EntityDc = EntityDc :: default () ; }
impl lgn_data_runtime::Resource for EntityDc {
    const TYPENAME: &'static str = "runtime_entitydc";
}
impl lgn_data_runtime::Asset for EntityDc {
    type Loader = EntityDcLoader;
}
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct EntityDcReferenceType(lgn_data_runtime::Reference<EntityDc>);
lgn_data_model::implement_primitive_type_def!(EntityDcReferenceType);
#[derive(Default)]
pub struct EntityDcLoader {}
impl lgn_data_runtime::AssetLoader for EntityDcLoader {
    fn load(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn std::any::Any + Send + Sync>> {
        let output: EntityDc = bincode::deserialize_from(reader).map_err(|_err| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to parse")
        })?;
        Ok(Box::new(output))
    }
    fn load_init(&mut self, _asset: &mut (dyn std::any::Any + Send + Sync)) {}
}
