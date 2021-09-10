// Types that will eventually moved to library crates

use legion_data_offline::resource::{Resource, ResourceId, ResourceProcessor, ResourceType};
use legion_math::prelude::*;
use serde::{Deserialize, Serialize};

// ------------------ Material -----------------------------------

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

// ------------------ Mesh -----------------------------------

pub const MESH_TYPE_ID: ResourceType = ResourceType::new(b"offline_mesh");

#[derive(Resource, Serialize, Deserialize)]
pub struct Mesh {
    pub sub_meshes: Vec<SubMesh>,
}

#[derive(Serialize, Deserialize)]
pub struct SubMesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u16>,
    pub material: ResourceId,
}

pub struct MeshProcessor {}

impl ResourceProcessor for MeshProcessor {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(Mesh {
            sub_meshes: Vec::default(),
        })
    }

    fn extract_build_dependencies(
        &mut self,
        _resource: &dyn Resource,
    ) -> Vec<legion_data_offline::asset::AssetPathId> {
        // let mesh = resource.downcast_ref::<Mesh>().unwrap();
        Vec::new()
    }

    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.downcast_ref::<Mesh>().unwrap();
        serde_json::to_writer(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Resource>> {
        let resource: Mesh = serde_json::from_reader(reader).unwrap();
        Ok(Box::new(resource))
    }
}
