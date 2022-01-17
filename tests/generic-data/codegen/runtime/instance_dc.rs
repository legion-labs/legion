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
    const TYPENAME: &'static str = "runtime_instancedc";
}
impl lgn_data_runtime::Asset for InstanceDc {
    type Loader = InstanceDcLoader;
}
#[derive(serde :: Serialize, serde :: Deserialize, PartialEq)]
pub struct InstanceDcReferenceType(lgn_data_runtime::Reference<InstanceDc>);
lgn_data_model::implement_primitive_type_def!(InstanceDcReferenceType);
#[derive(Default)]
pub struct InstanceDcLoader {}
impl lgn_data_runtime::AssetLoader for InstanceDcLoader {
    fn load(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn std::any::Any + Send + Sync>> {
        let output: InstanceDc = bincode::deserialize_from(reader).map_err(|_err| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to parse")
        })?;
        Ok(Box::new(output))
    }
    fn load_init(&mut self, _asset: &mut (dyn std::any::Any + Send + Sync)) {}
}
