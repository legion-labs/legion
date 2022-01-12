#[derive(serde :: Serialize, serde :: Deserialize, PartialEq)]
pub struct InstanceDc {}
impl InstanceDc {
    #[allow(dead_code)]
    const SIGNATURE_HASH: u64 = 15261495266635127007u64;
    #[allow(dead_code)]
    pub fn get_default_instance() -> &'static Self {
        &__INSTANCEDC_DEFAULT
    }
}
#[allow(clippy::derivable_impls)]
impl Default for InstanceDc {
    fn default() -> Self {
        Self {}
    }
}
impl lgn_data_model::TypeReflection for InstanceDc {
    fn get_type(&self) -> lgn_data_model::TypeDefinition {
        Self::get_type_def()
    }
    #[allow(unused_mut)]
    #[allow(clippy::let_and_return)]
    #[allow(clippy::too_many_lines)]
    fn get_type_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_struct_descriptor!(InstanceDc, vec![]);
        lgn_data_model::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_option_descriptor!(InstanceDc);
        lgn_data_model::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_array_descriptor!(InstanceDc);
        lgn_data_model::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { # [allow (clippy :: needless_update)] static ref __INSTANCEDC_DEFAULT : InstanceDc = InstanceDc :: default () ; }
impl lgn_data_runtime::Resource for InstanceDc {
    const TYPENAME: &'static str = "offline_instancedc";
}
impl lgn_data_runtime::Asset for InstanceDc {
    type Loader = InstanceDcProcessor;
}
impl lgn_data_offline::resource::OfflineResource for InstanceDc {
    type Processor = InstanceDcProcessor;
}
#[derive(Default)]
pub struct InstanceDcProcessor {}
impl lgn_data_runtime::AssetLoader for InstanceDcProcessor {
    fn load(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn std::any::Any + Send + Sync>> {
        let mut instance = InstanceDc::default();
        let values: serde_json::Value = serde_json::from_reader(reader)
            .map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
        lgn_data_model::json_utils::reflection_apply_json_edit::<InstanceDc>(
            &mut instance,
            &values,
        )
        .map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
        Ok(Box::new(instance))
    }
    fn load_init(&mut self, _asset: &mut (dyn std::any::Any + Send + Sync)) {}
}
impl lgn_data_offline::resource::ResourceProcessor for InstanceDcProcessor {
    fn new_resource(&mut self) -> Box<dyn std::any::Any + Send + Sync> {
        Box::new(InstanceDc::default())
    }
    fn extract_build_dependencies(
        &mut self,
        resource: &dyn std::any::Any,
    ) -> Vec<lgn_data_offline::ResourcePathId> {
        let instance = resource.downcast_ref::<InstanceDc>().unwrap();
        lgn_data_offline::extract_resource_dependencies(instance)
            .unwrap_or_default()
            .into_iter()
            .collect()
    }
    fn get_resource_type_name(&self) -> Option<&'static str> {
        Some(<InstanceDc as lgn_data_runtime::Resource>::TYPENAME)
    }
    fn write_resource(
        &mut self,
        resource: &dyn std::any::Any,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let instance = resource.downcast_ref::<InstanceDc>().unwrap();
        let values = lgn_data_model::json_utils::reflection_save_relative_json(
            instance,
            InstanceDc::get_default_instance(),
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
        if let Some(instance) = resource.downcast_ref::<InstanceDc>() {
            return Some(instance);
        }
        None
    }
    fn get_resource_reflection_mut<'a>(
        &self,
        resource: &'a mut dyn std::any::Any,
    ) -> Option<&'a mut dyn lgn_data_model::TypeReflection> {
        if let Some(instance) = resource.downcast_mut::<InstanceDc>() {
            return Some(instance);
        }
        None
    }
}
