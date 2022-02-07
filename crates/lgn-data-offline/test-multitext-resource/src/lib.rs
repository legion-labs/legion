use std::{any::Any, io};

use lgn_data_offline::{
    resource::{OfflineResource, ResourceProcessor, ResourceProcessorError},
    ResourcePathId,
};
use lgn_data_runtime::{resource, Asset, AssetLoader, AssetLoaderError, Resource};
use serde::{Deserialize, Serialize};

#[resource("multitext_resource")]
#[derive(Serialize, Deserialize)]
pub struct MultiTextResource {
    pub text_list: Vec<String>,
}

impl Asset for MultiTextResource {
    type Loader = MultiTextResourceProc;
}

impl OfflineResource for MultiTextResource {
    type Processor = MultiTextResourceProc;
}

#[derive(Default)]
pub struct MultiTextResourceProc {}

impl AssetLoader for MultiTextResourceProc {
    fn load(
        &mut self,
        reader: &mut dyn io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, AssetLoaderError> {
        let resource: MultiTextResource = serde_json::from_reader(reader).unwrap();
        let boxed = Box::new(resource);
        Ok(boxed)
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}

impl ResourceProcessor for MultiTextResourceProc {
    fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
        Box::new(MultiTextResource { text_list: vec![] })
    }

    fn extract_build_dependencies(&mut self, _resource: &dyn Any) -> Vec<ResourcePathId> {
        vec![]
    }

    fn write_resource(
        &self,
        resource: &dyn Any,
        writer: &mut dyn std::io::Write,
    ) -> Result<usize, ResourceProcessorError> {
        let resource = resource.downcast_ref::<MultiTextResource>().unwrap();
        serde_json::to_writer_pretty(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, ResourceProcessorError> {
        Ok(self.load(reader)?)
    }
}
