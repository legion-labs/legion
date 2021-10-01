//! A module providing offline material related functionality.

use legion_data_offline::{
    asset::AssetPathId,
    resource::{Resource, ResourceProcessor},
};

use legion_data_runtime::ResourceType;
use serde::{Deserialize, Serialize};

/// Type id.
pub const TYPE_ID: ResourceType = ResourceType::new(b"offline_material");

/// Offline material resource.
#[derive(Resource, Default, Serialize, Deserialize)]
pub struct Material {
    /// Albedo texture reference.
    pub albedo: Option<AssetPathId>,
    /// Normal texture reference.
    pub normal: Option<AssetPathId>,
    /// Roughness texture reference.
    pub roughness: Option<AssetPathId>,
    /// Metalness texture reference.
    pub metalness: Option<AssetPathId>,
}

/// Processor of [`Material`]
#[derive(Default)]
pub struct MaterialProcessor {}

impl ResourceProcessor for MaterialProcessor {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(Material::default())
    }

    fn extract_build_dependencies(&mut self, resource: &dyn Resource) -> Vec<AssetPathId> {
        let material = resource.downcast_ref::<Material>().unwrap();
        let mut deps = vec![];
        if let Some(path) = &material.albedo {
            deps.push(path.clone());
        }
        if let Some(path) = &material.normal {
            deps.push(path.clone());
        }
        if let Some(path) = &material.roughness {
            deps.push(path.clone());
        }
        if let Some(path) = &material.metalness {
            deps.push(path.clone());
        }
        deps
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
        let result: Result<Material, serde_json::Error> = serde_json::from_reader(reader);
        match result {
            Ok(resource) => Ok(Box::new(resource)),
            Err(json_err) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                json_err.to_string(),
            )),
        }
    }
}
