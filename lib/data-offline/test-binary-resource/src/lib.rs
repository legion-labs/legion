use std::any::Any;

use legion_data_offline::{
    resource::{OfflineResource, ResourceProcessor},
    ResourcePathId,
};

use legion_data_runtime::{resource, Resource};
use serde::{Deserialize, Serialize};

#[resource("bin")]
#[derive(Serialize, Deserialize)]
pub struct BinaryResource {
    pub content: Vec<u8>,
}

impl OfflineResource for BinaryResource {
    type Processor = BinaryResourceProc;
}

#[derive(Default)]
pub struct BinaryResourceProc {}

impl ResourceProcessor for BinaryResourceProc {
    fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
        Box::new(BinaryResource { content: vec![] })
    }

    fn extract_build_dependencies(&mut self, _resource: &dyn Any) -> Vec<ResourcePathId> {
        vec![]
    }

    fn write_resource(
        &mut self,
        resource: &dyn Any,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.downcast_ref::<BinaryResource>().unwrap();
        writer.write_all(&resource.content)?;
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(&mut self, reader: &mut dyn std::io::Read) -> std::io::Result<Box<dyn Any + Send + Sync>> {
        let mut resource = BinaryResource { content: vec![] };
        reader.read_to_end(&mut resource.content)?;
        let boxed = Box::new(resource);
        Ok(boxed)
    }
}
