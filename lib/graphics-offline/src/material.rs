//! A module providing offline material related functionality.

use std::{any::Any, io};

use lgn_data_offline::{
    resource::{OfflineResource, ResourceProcessor},
    ResourcePathId,
};
use lgn_data_runtime::{resource, Asset, AssetLoader, Resource};
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

impl Asset for Material {
    type Loader = MaterialProcessor;
}

impl OfflineResource for Material {
    type Processor = MaterialProcessor;
}

/// Processor of [`Material`]
#[derive(Default)]
pub struct MaterialProcessor {}

impl AssetLoader for MaterialProcessor {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let result: Material = serde_json::from_reader(reader)?;
        Ok(Box::new(result))
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}

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

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Any + Send + Sync>> {
        self.load(reader)
    }
}
