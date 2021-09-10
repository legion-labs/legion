use super::raw_data;
use crate::offline_data;

impl From<raw_data::Material> for offline_data::Material {
    fn from(raw_material: raw_data::Material) -> Self {
        Self {
            albedo: raw_material.albedo,
            normal: raw_material.normal,
            roughness: raw_material.roughness,
            metalness: raw_material.metalness,
        }
    }
}
