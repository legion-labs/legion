use std::any::Any;

use legion_data_offline::{
    resource::{OfflineResource, ResourceProcessor},
    ResourcePathId,
};
use legion_data_runtime::{resource, Resource};

use serde::{Deserialize, Serialize};

#[resource("text")]
#[derive(Serialize, Deserialize)]
pub struct TextResource {
    pub content: String,
}

impl OfflineResource for TextResource {
    type Processor = TextResourceProc;
}

#[derive(Default)]
pub struct TextResourceProc {}

impl ResourceProcessor for TextResourceProc {
    fn new_resource(&mut self) -> Box<dyn Any> {
        Box::new(TextResource {
            content: String::from("7"),
        })
    }

    fn extract_build_dependencies(&mut self, _resource: &dyn Any) -> Vec<ResourcePathId> {
        vec![]
    }

    fn write_resource(
        &mut self,
        resource: &dyn Any,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.downcast_ref::<TextResource>().unwrap();
        serde_json::to_writer(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(&mut self, reader: &mut dyn std::io::Read) -> std::io::Result<Box<dyn Any>> {
        let resource: TextResource = serde_json::from_reader(reader).unwrap();
        let boxed = Box::new(resource);
        Ok(boxed)
    }
}
