//! A module providing runtime material related functionality.

use std::{any::Any, io, sync::Arc};

use byteorder::{LittleEndian, ReadBytesExt};
use lgn_data_runtime::{
    resource, Asset, AssetLoader, AssetRegistry, Reference, Resource, ResourceId, ResourceType,
    ResourceTypeAndId,
};
use lgn_tracing::info;

use crate::Texture;

/// Runtime material.
#[resource("runtime_material")]
pub struct Material {
    /// Albedo texture reference.
    pub albedo: Reference<Texture>,
    /// Normal texture reference.
    pub normal: Reference<Texture>,
    /// Roughness texture reference.
    pub roughness: Reference<Texture>,
    /// Metalness texture reference.
    pub metalness: Reference<Texture>,
    /// Diffuse or metal surface color.
    pub base_color: (f32, f32, f32, f32),
    /// lends between a non-metallic and metallic material model
    pub metallic: f32,
    /// Amount of dielectric specular reflection. Specifies facing (along normal) reflectivity in the most common 0 - 8% range.
    pub specular: f32,
    /// Specifies microfacet roughness of the surface for diffuse and specular reflection.
    pub roughness_value: f32,
}

impl Asset for Material {
    type Loader = MaterialLoader;
}

/// Creator of [`Material`].
#[derive(Default)]
pub struct MaterialLoader {
    registry: Option<Arc<AssetRegistry>>,
}

fn read_asset_id<T>(reader: &mut dyn std::io::Read) -> Result<Reference<T>, std::io::Error>
where
    T: Any + Resource,
{
    let underlying_type = reader.read_u64::<LittleEndian>()?;
    let underlying_id = reader.read_u128::<LittleEndian>()?;
    Ok(Reference::Passive(ResourceTypeAndId {
        kind: ResourceType::from_raw(underlying_type),
        id: ResourceId::from_raw(underlying_id),
    }))
}

fn read_with_default(reader: &mut dyn io::Read, default: f32) -> f32 {
    reader
        .read_f32::<LittleEndian>()
        .or::<f32>(Ok(default))
        .unwrap()
}

impl AssetLoader for MaterialLoader {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let albedo = read_asset_id(reader)?;
        let normal = read_asset_id(reader)?;
        let roughness = read_asset_id(reader)?;
        let metalness = read_asset_id(reader)?;

        let r = read_with_default(reader, 0.8);
        let g = read_with_default(reader, 0.8);
        let b = read_with_default(reader, 0.8);
        let a = read_with_default(reader, 1.0);

        let output = Material {
            albedo,
            normal,
            roughness,
            metalness,
            base_color: (r, g, b, a),
            metallic: read_with_default(reader, 0.0),
            specular: read_with_default(reader, 0.5),
            roughness_value: read_with_default(reader, 0.4),
        };

        Ok(Box::new(output))
    }

    fn load_init(&mut self, asset: &mut (dyn Any + Send + Sync)) {
        let material = asset.downcast_mut::<Material>().unwrap();
        info!("runtime material loaded");

        // activate references
        if let Some(registry) = &self.registry {
            material.albedo.activate(registry);
            material.normal.activate(registry);
            material.roughness.activate(registry);
            material.metalness.activate(registry);
        }
    }

    fn register_registry(&mut self, registry: Arc<AssetRegistry>) {
        self.registry = Some(registry);
    }
}
