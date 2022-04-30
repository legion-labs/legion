use std::{any::Any, io};

use lgn_data_runtime::{
    resource, Asset, AssetLoader, AssetLoaderError, OfflineResource, Resource, ResourcePathId,
    ResourceProcessor, ResourceProcessorError,
};
use serde::{Deserialize, Serialize};

#[resource("bin")]
#[derive(Serialize, Deserialize, Clone)]
pub struct BinaryResource {
    pub content: Vec<u8>,
}

impl Asset for BinaryResource {
    type Loader = BinaryResourceProc;
}

impl OfflineResource for BinaryResource {
    type Processor = BinaryResourceProc;
}

#[derive(Default)]
pub struct BinaryResourceProc {}

impl AssetLoader for BinaryResourceProc {
    fn load(
        &mut self,
        reader: &mut dyn io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, AssetLoaderError> {
        let mut resource = BinaryResource { content: vec![] };
        reader.read_to_end(&mut resource.content)?;
        let boxed = Box::new(resource);
        Ok(boxed)
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}

impl ResourceProcessor for BinaryResourceProc {
    fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
        Box::new(BinaryResource { content: vec![] })
    }

    fn extract_build_dependencies(&mut self, _resource: &dyn Any) -> Vec<ResourcePathId> {
        vec![]
    }

    fn write_resource(
        &self,
        resource: &dyn Any,
        writer: &mut dyn std::io::Write,
    ) -> Result<usize, ResourceProcessorError> {
        let resource = resource.downcast_ref::<BinaryResource>().unwrap();
        writer.write_all(&resource.content)?;
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, ResourceProcessorError> {
        Ok(self.load(reader)?)
    }
}
