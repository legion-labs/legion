#![allow(dead_code)]
#![allow(clippy::needless_update)]

use lgn_data_runtime::Component;
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct EntityDc {
    pub name: String,
    pub components: Vec<Box<dyn Component>>,
}
impl EntityDc {
    const SIGNATURE_HASH: u64 = 2369000756644127862u64;
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
impl lgn_data_reflection::TypeReflection for EntityDc {
    fn get_type(&self) -> lgn_data_reflection::TypeDefinition {
        Self::get_type_def()
    }
    fn get_type_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection :: implement_struct_descriptor ! (EntityDc , vec ! [lgn_data_reflection :: FieldDescriptor { field_name : "name" . into () , offset : memoffset :: offset_of ! (EntityDc , name) , field_type : < String as lgn_data_reflection :: TypeReflection > :: get_type_def () , group : "" . into () } , lgn_data_reflection :: FieldDescriptor { field_name : "components" . into () , offset : memoffset :: offset_of ! (EntityDc , components) , field_type : < Vec < Box < dyn Component > > as lgn_data_reflection :: TypeReflection > :: get_type_def () , group : "" . into () } ,]);
        lgn_data_reflection::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_option_descriptor!(EntityDc);
        lgn_data_reflection::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_array_descriptor!(EntityDc);
        lgn_data_reflection::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { static ref __ENTITYDC_DEFAULT : EntityDc = EntityDc { .. EntityDc :: default () } ; }
use lgn_data_offline::resource::{OfflineResource, ResourceProcessor};
use lgn_data_runtime::{Asset, AssetLoader, Resource};
use std::{any::Any, io};
impl Resource for EntityDc {
    const TYPENAME: &'static str = "offline_entitydc";
}
impl Asset for EntityDc {
    type Loader = EntityDcProcessor;
}
impl OfflineResource for EntityDc {
    type Processor = EntityDcProcessor;
}
#[derive(Default)]
pub struct EntityDcProcessor {}
impl AssetLoader for EntityDcProcessor {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let mut instance = EntityDc {
            ..EntityDc::default()
        };
        let values: serde_json::Value = serde_json::from_reader(reader)
            .map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
        lgn_data_reflection::json_utils::reflection_apply_json_edit::<EntityDc>(
            &mut instance,
            &values,
        )
        .map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
        Ok(Box::new(instance))
    }
    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}
impl ResourceProcessor for EntityDcProcessor {
    fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
        Box::new(EntityDc {
            ..EntityDc::default()
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
        let instance = resource.downcast_ref::<EntityDc>().unwrap();
        let values = lgn_data_reflection::json_utils::reflection_save_relative_json(
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
    ) -> std::io::Result<Box<dyn Any + Send + Sync>> {
        self.load(reader)
    }
    fn get_resource_reflection<'a>(
        &self,
        resource: &'a dyn Any,
    ) -> Option<&'a dyn lgn_data_reflection::TypeReflection> {
        if let Some(instance) = resource.downcast_ref::<EntityDc>() {
            return Some(instance);
        }
        None
    }
    fn get_resource_reflection_mut<'a>(
        &self,
        resource: &'a mut dyn Any,
    ) -> Option<&'a mut dyn lgn_data_reflection::TypeReflection> {
        if let Some(instance) = resource.downcast_mut::<EntityDc>() {
            return Some(instance);
        }
        None
    }
}
