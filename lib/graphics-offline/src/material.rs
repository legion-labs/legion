//! A module providing offline material related functionality.

use std::any::Any;

use legion_data_offline::{
    resource::{OfflineResource, ResourceProcessor},
    ResourcePathId,
};
use legion_data_runtime::{resource, Resource};
use serde::{Deserialize, Serialize};

/// Offline material resource.
#[resource("offline_material")]
#[derive(Default, Serialize, Deserialize)]
pub struct Material {
    /// Albedo texture reference.
    pub albedo: Option<ResourcePathId>,
    /// Normal texture reference.
    pub normal: Option<ResourcePathId>,
    /// Roughness texture reference.
    pub roughness: Option<ResourcePathId>,
    /// Metalness texture reference.
    pub metalness: Option<ResourcePathId>,
}

impl OfflineResource for Material {
    type Processor = MaterialProcessor;
}

/// Processor of [`Material`]
#[derive(Default)]
pub struct MaterialProcessor {}

impl ResourceProcessor for MaterialProcessor {
    fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
        Box::new(Material::default())
    }

    fn extract_build_dependencies(&mut self, resource: &dyn Any) -> Vec<ResourcePathId> {
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
        resource: &dyn Any,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.downcast_ref::<Material>().unwrap();
        serde_json::to_writer(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(&mut self, reader: &mut dyn std::io::Read) -> std::io::Result<Box<dyn Any + Send + Sync>> {
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
