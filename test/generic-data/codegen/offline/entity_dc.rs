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
    const TYPENAME: &'static str = "offline_entitydc";
}
impl lgn_data_runtime::Asset for EntityDc {
    type Loader = EntityDcProcessor;
}
impl lgn_data_offline::resource::OfflineResource for EntityDc {
    type Processor = EntityDcProcessor;
}
#[derive(Default)]
pub struct EntityDcProcessor {}
impl lgn_data_runtime::AssetLoader for EntityDcProcessor {
    fn load(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn std::any::Any + Send + Sync>> {
        let mut instance = EntityDc::default();
        let values: serde_json::Value = serde_json::from_reader(reader)
            .map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
        lgn_data_model::json_utils::reflection_apply_json_edit::<EntityDc>(&mut instance, &values)
            .map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
        Ok(Box::new(instance))
    }
    fn load_init(&mut self, _asset: &mut (dyn std::any::Any + Send + Sync)) {}
}
impl lgn_data_offline::resource::ResourceProcessor for EntityDcProcessor {
    fn new_resource(&mut self) -> Box<dyn std::any::Any + Send + Sync> {
        Box::new(EntityDc::default())
    }
    fn extract_build_dependencies(
        &mut self,
        _resource: &dyn std::any::Any,
    ) -> Vec<lgn_data_offline::ResourcePathId> {
        vec![]
    }
    fn get_resource_type_name(&self) -> Option<&'static str> {
        Some(<EntityDc as lgn_data_runtime::Resource>::TYPENAME)
    }
    fn write_resource(
        &mut self,
        resource: &dyn std::any::Any,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let instance = resource.downcast_ref::<EntityDc>().unwrap();
        let values = lgn_data_model::json_utils::reflection_save_relative_json(
            instance,
            EntityDc::get_default_instance(),
        )
        .map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
        serde_json::to_writer_pretty(writer, &values)
            .map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
        Ok(1)
    }
    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn std::any::Any + Send + Sync>> {
        use lgn_data_runtime::AssetLoader;
        self.load(reader)
    }
    fn get_resource_reflection<'a>(
        &self,
        resource: &'a dyn std::any::Any,
    ) -> Option<&'a dyn lgn_data_model::TypeReflection> {
        if let Some(instance) = resource.downcast_ref::<EntityDc>() {
            return Some(instance);
        }
        None
    }
    fn get_resource_reflection_mut<'a>(
        &self,
        resource: &'a mut dyn std::any::Any,
    ) -> Option<&'a mut dyn lgn_data_model::TypeReflection> {
        if let Some(instance) = resource.downcast_mut::<EntityDc>() {
            return Some(instance);
        }
        None
    }
}
