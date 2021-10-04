use std::any::Any;

use legion_data_offline::{
    resource::{OfflineResource, ResourceProcessor},
    ResourcePathId,
};

use legion_data_runtime::{resource, Resource};
use serde::{Deserialize, Serialize};

#[resource("multitext_resource")]
#[derive(Serialize, Deserialize)]
pub struct MultiTextResource {
    pub text_list: Vec<String>,
}

impl OfflineResource for MultiTextResource {
    type Processor = MultiTextResourceProc;
}

#[derive(Default)]
pub struct MultiTextResourceProc {}

impl ResourceProcessor for MultiTextResourceProc {
    fn new_resource(&mut self) -> Box<dyn Any> {
        Box::new(MultiTextResource { text_list: vec![] })
    }

    fn extract_build_dependencies(&mut self, _resource: &dyn Any) -> Vec<ResourcePathId> {
        vec![]
    }

    fn write_resource(
        &mut self,
        resource: &dyn Any,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.downcast_ref::<MultiTextResource>().unwrap();
        serde_json::to_writer(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(&mut self, reader: &mut dyn std::io::Read) -> std::io::Result<Box<dyn Any>> {
        let resource: MultiTextResource = serde_json::from_reader(reader).unwrap();
        let boxed = Box::new(resource);
        Ok(boxed)
    }
}
