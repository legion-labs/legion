use std::io;

use lgn_data_runtime::{
    resource, Asset, AssetLoader, AssetLoaderError, Metadata, OfflineResource, Resource,
    ResourceDescriptor, ResourcePathId, ResourcePathName, ResourceProcessor,
    ResourceProcessorError,
};
use serde::{Deserialize, Serialize};

#[resource("text")]
#[derive(Serialize, Deserialize, Clone)]
pub struct TextResource {
    pub meta: Metadata,
    pub content: String,
}

impl Asset for TextResource {
    type Loader = TextResourceProc;
}

impl OfflineResource for TextResource {
    type Processor = TextResourceProc;
}

#[derive(Default)]
pub struct TextResourceProc {}

impl AssetLoader for TextResourceProc {
    fn load(&mut self, reader: &mut dyn io::Read) -> Result<Box<dyn Resource>, AssetLoaderError> {
        let resource: TextResource = serde_json::from_reader(reader).unwrap();
        let boxed = Box::new(resource);
        Ok(boxed)
    }

    fn load_init(&mut self, _asset: &mut (dyn Resource)) {}
}

impl ResourceProcessor for TextResourceProc {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(TextResource {
            meta: Metadata::new(
                ResourcePathName::default(),
                TextResource::TYPENAME,
                TextResource::TYPE,
            ),
            content: String::from("7"),
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
        let resource = resource.downcast_ref::<TextResource>().unwrap();
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
