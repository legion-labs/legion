#![allow(dead_code)]
#![allow(clippy::needless_update)]

#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct InstanceDc {}
impl InstanceDc {
    const SIGNATURE_HASH: u64 = 15261495266635127007u64;
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
impl lgn_data_reflection::TypeReflection for InstanceDc {
    fn get_type(&self) -> lgn_data_reflection::TypeDefinition {
        Self::get_type_def()
    }
    fn get_type_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_struct_descriptor!(InstanceDc, vec![]);
        lgn_data_reflection::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_option_descriptor!(InstanceDc);
        lgn_data_reflection::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_array_descriptor!(InstanceDc);
        lgn_data_reflection::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { static ref __INSTANCEDC_DEFAULT : InstanceDc = InstanceDc { .. InstanceDc :: default () } ; }
use lgn_data_offline::resource::{OfflineResource, ResourceProcessor};
use lgn_data_runtime::{Asset, AssetLoader, Resource};
use std::{any::Any, io};
impl Resource for InstanceDc {
    const TYPENAME: &'static str = "offline_instancedc";
}
impl Asset for InstanceDc {
    type Loader = InstanceDcProcessor;
}
impl OfflineResource for InstanceDc {
    type Processor = InstanceDcProcessor;
}
#[derive(Default)]
pub struct InstanceDcProcessor {}
impl AssetLoader for InstanceDcProcessor {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let mut instance = InstanceDc {
            ..InstanceDc::default()
        };
        let values: serde_json::Value = serde_json::from_reader(reader)
            .map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
        lgn_data_reflection::json_utils::reflection_apply_json_edit::<InstanceDc>(
            &mut instance,
            &values,
        )
        .map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
        Ok(Box::new(instance))
    }
    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}
impl ResourceProcessor for InstanceDcProcessor {
    fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
        Box::new(InstanceDc {
            ..InstanceDc::default()
        })
    }
    fn extract_build_dependencies(
        &mut self,
        _resource: &dyn Any,
    ) -> Vec<lgn_data_offline::ResourcePathId> {
        vec![]
    }
    #[allow(clippy::float_cmp, clippy::too_many_lines)]
    fn write_resource(
        &mut self,
        resource: &dyn Any,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let instance = resource.downcast_ref::<InstanceDc>().unwrap();
        let values = lgn_data_reflection::json_utils::reflection_save_relative_json(
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
    ) -> std::io::Result<Box<dyn Any + Send + Sync>> {
        self.load(reader)
    }
    fn get_resource_reflection<'a>(
        &self,
        resource: &'a dyn Any,
    ) -> Option<&'a dyn lgn_data_reflection::TypeReflection> {
        if let Some(instance) = resource.downcast_ref::<InstanceDc>() {
            return Some(instance);
        }
        None
    }
    fn get_resource_reflection_mut<'a>(
        &self,
        resource: &'a mut dyn Any,
    ) -> Option<&'a mut dyn lgn_data_reflection::TypeReflection> {
        if let Some(instance) = resource.downcast_mut::<InstanceDc>() {
            return Some(instance);
        }
        None
    }
}
