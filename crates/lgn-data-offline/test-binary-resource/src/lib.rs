use std::io;

use lgn_data_runtime::{
    resource, Asset, AssetLoader, AssetLoaderError, Metadata, OfflineResource, Resource,
    ResourceDescriptor, ResourcePathId, ResourcePathName, ResourceProcessor,
    ResourceProcessorError,
};
use serde::{Deserialize, Serialize};

/// This is the main resource.
#[resource("bin")]
#[derive(Serialize, Deserialize, Clone)]
pub struct BinaryResource {
    pub meta: Metadata,

    #[serde(with = "serde_bytes")]
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
    fn load(&mut self, reader: &mut dyn io::Read) -> Result<Box<dyn Resource>, AssetLoaderError> {
        let resource: BinaryResource = serde_json::from_reader(reader).unwrap();
        Ok(Box::new(resource))
    }

    fn load_init(&mut self, _asset: &mut (dyn Resource)) {}
}

impl ResourceProcessor for BinaryResourceProc {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(BinaryResource {
            meta: Metadata::new(
                ResourcePathName::default(),
                BinaryResource::TYPENAME,
                BinaryResource::TYPE,
            ),
            content: vec![],
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
        let resource = resource.downcast_ref::<BinaryResource>().unwrap();
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
