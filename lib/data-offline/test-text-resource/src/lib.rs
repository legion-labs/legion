use legion_data_offline::{
    asset::AssetPathId,
    resource::{Resource, ResourceProcessor, ResourceType},
};

use serde::{Deserialize, Serialize};

pub const TYPE_ID: ResourceType = ResourceType::new(b"text_resource");

#[derive(Resource, Serialize, Deserialize)]
pub struct TextResource {
    pub content: String,
}

pub struct TextResourceProc {}

impl ResourceProcessor for TextResourceProc {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(TextResource {
            content: String::from("7"),
        })
    }

    fn extract_build_dependencies(&mut self, _resource: &dyn Resource) -> Vec<AssetPathId> {
        vec![]
    }

    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.downcast_ref::<TextResource>().unwrap();
        serde_json::to_writer(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Resource>> {
        let resource: TextResource = serde_json::from_reader(reader).unwrap();
        let boxed = Box::new(resource);
        Ok(boxed)
    }
}
