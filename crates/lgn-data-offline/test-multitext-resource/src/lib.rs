use std::io;

use lgn_data_runtime::{
    resource, Asset, AssetLoader, AssetLoaderError, Metadata, OfflineResource, Resource,
    ResourceDescriptor, ResourcePathId, ResourcePathName, ResourceProcessor,
    ResourceProcessorError,
};
use serde::{Deserialize, Serialize};

#[resource("multitext_resource")]
#[derive(Serialize, Deserialize, Clone)]
pub struct MultiTextResource {
    pub meta: Metadata,
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
    fn load(&mut self, reader: &mut dyn io::Read) -> Result<Box<dyn Resource>, AssetLoaderError> {
        let resource: MultiTextResource = serde_json::from_reader(reader).unwrap();
        Ok(Box::new(resource))
    }

    fn load_init(&mut self, _asset: &mut (dyn Resource)) {}
}

impl ResourceProcessor for MultiTextResourceProc {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(MultiTextResource {
            meta: Metadata::new(
                ResourcePathName::default(),
                MultiTextResource::TYPENAME,
                MultiTextResource::TYPE,
            ),
            text_list: vec![],
        })
    }

    fn extract_build_dependencies(&mut self, _resource: &dyn Resource) -> Vec<ResourcePathId> {
        vec![]
    }

    fn write_resource(
        &self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> Result<usize, ResourceProcessorError> {
        let resource = resource.downcast_ref::<MultiTextResource>().unwrap();
        serde_json::to_writer_pretty(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Resource>, ResourceProcessorError> {
        Ok(self.load(reader)?)
    }
}
