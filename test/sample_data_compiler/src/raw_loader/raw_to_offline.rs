use super::raw_data;
use crate::offline_data;

impl From<raw_data::Material> for offline_data::Material {
    fn from(raw: raw_data::Material) -> Self {
        Self {
            albedo: raw.albedo,
            normal: raw.normal,
            roughness: raw.roughness,
            metalness: raw.metalness,
        }
    }
}

impl From<raw_data::Mesh> for offline_data::Mesh {
    fn from(raw: raw_data::Mesh) -> Self {
        let mut sub_meshes: Vec<offline_data::SubMesh> = Vec::new();
        for sub_mesh in raw.sub_meshes {
            sub_meshes.push(offline_data::SubMesh {
                positions: sub_mesh.positions.clone(),
                normals: sub_mesh.normals.clone(),
                uvs: sub_mesh.uvs.clone(),
                indices: sub_mesh.indices.clone(),
                material: sub_mesh.material,
            });
        }
        Self { sub_meshes }
    }
}
