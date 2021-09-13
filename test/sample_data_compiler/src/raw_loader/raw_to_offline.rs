use std::collections::HashMap;

use legion_data_offline::resource::{ResourceId, ResourcePathName};

use super::raw_data;
use crate::offline_data;

pub trait FromRaw<T> {
    fn from_raw(raw: T, references: &HashMap<ResourcePathName, ResourceId>) -> Self;
}

impl FromRaw<raw_data::Material> for offline_data::Material {
    fn from_raw(
        raw: raw_data::Material,
        _references: &HashMap<ResourcePathName, ResourceId>,
    ) -> Self {
        Self {
            albedo: raw.albedo,
            normal: raw.normal,
            roughness: raw.roughness,
            metalness: raw.metalness,
        }
    }
}

impl FromRaw<raw_data::Mesh> for offline_data::Mesh {
    fn from_raw(raw: raw_data::Mesh, references: &HashMap<ResourcePathName, ResourceId>) -> Self {
        let mut sub_meshes: Vec<offline_data::SubMesh> = Vec::new();
        for sub_mesh in raw.sub_meshes {
            sub_meshes.push(offline_data::SubMesh {
                positions: sub_mesh.positions.clone(),
                normals: sub_mesh.normals.clone(),
                uvs: sub_mesh.uvs.clone(),
                indices: sub_mesh.indices.clone(),
                material: *references
                    .get(&ResourcePathName::new(sub_mesh.material))
                    .unwrap(),
            });
        }
        Self { sub_meshes }
    }
}
