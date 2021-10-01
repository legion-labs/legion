use legion_data_offline::{
    resource::{Resource, ResourceProcessor},
    ResourcePathId,
};

use legion_data_runtime::ResourceType;
use serde::{Deserialize, Serialize};

pub const TYPE_ID: ResourceType = ResourceType::new(b"binary_resource");

#[derive(Resource, Serialize, Deserialize)]
pub struct BinaryResource {
    pub content: Vec<u8>,
}

pub struct BinaryResourceProc {}

impl ResourceProcessor for BinaryResourceProc {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(BinaryResource { content: vec![] })
    }

    fn extract_build_dependencies(&mut self, _resource: &dyn Resource) -> Vec<ResourcePathId> {
        vec![]
    }

    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.downcast_ref::<BinaryResource>().unwrap();
        writer.write_all(&resource.content)?;
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Resource>> {
        let mut resource = BinaryResource { content: vec![] };
        reader.read_to_end(&mut resource.content)?;
        let boxed = Box::new(resource);
        Ok(boxed)
    }
}
