// Types that will eventually moved to library crates

use legion_data_offline::resource::{Resource, ResourceProcessor, ResourceType};
use serde::{Deserialize, Serialize};

pub const MATERIAL_TYPE_ID: ResourceType = ResourceType::new(b"offline_material");

#[derive(Resource, Serialize, Deserialize)]
pub struct Material {
    pub albedo: TextureReference,
    pub normal: TextureReference,
    pub roughness: TextureReference,
    pub metalness: TextureReference,
}

pub struct MaterialProcessor {}

impl ResourceProcessor for MaterialProcessor {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(Material {
            albedo: TextureReference::default(),
            normal: TextureReference::default(),
            roughness: TextureReference::default(),
            metalness: TextureReference::default(),
        })
    }

    fn extract_build_dependencies(
        &mut self,
        _resource: &dyn Resource,
    ) -> Vec<legion_data_offline::asset::AssetPathId> {
        // let material = resource.downcast_ref::<Material>().unwrap();
        Vec::new()
    }

    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.downcast_ref::<Material>().unwrap();
        serde_json::to_writer(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Resource>> {
        let resource: Material = serde_json::from_reader(reader).unwrap();
        Ok(Box::new(resource))
    }
}

pub type TextureReference = String;
