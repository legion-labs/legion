use std::collections::HashMap;

use legion_data_offline::resource::{ResourceId, ResourcePathName};

use crate::offline_data;

use super::raw_data;

pub trait FromRaw<T> {
    fn from_raw(raw: T, references: &HashMap<ResourcePathName, ResourceId>) -> Self;
}

fn lookup_reference(
    references: &HashMap<ResourcePathName, ResourceId>,
    path: &str,
) -> Option<ResourceId> {
    let path = ResourcePathName::new(path);
    references.get(&path).copied()
}

// ----- Entity conversions -----

impl FromRaw<raw_data::Entity> for offline_data::Entity {
    fn from_raw(raw: raw_data::Entity, references: &HashMap<ResourcePathName, ResourceId>) -> Self {
        let children: Vec<ResourceId> = raw
            .children
            .iter()
            .flat_map(|path| lookup_reference(references, path))
            .collect();
        let parent = match raw.parent {
            Some(parent) => lookup_reference(references, &parent),
            None => None,
        };
        let mut components: Vec<Box<dyn offline_data::Component>> = Vec::new();
        for component in raw.components {
            match component {
                raw_data::Component::Transform(raw) => {
                    components.push(Box::new(Into::<offline_data::Transform>::into(raw)));
                }
                raw_data::Component::Visual(raw) => {
                    components.push(Box::new(Into::<offline_data::Visual>::into(raw)));
                }
                raw_data::Component::GlobalIllumination(raw) => {
                    components.push(Box::new(Into::<offline_data::GlobalIllumination>::into(
                        raw,
                    )));
                }
                raw_data::Component::Navmesh(raw) => {
                    components.push(Box::new(Into::<offline_data::NavMesh>::into(raw)));
                }
                raw_data::Component::View(raw) => {
                    components.push(Box::new(Into::<offline_data::View>::into(raw)));
                }
                raw_data::Component::Light(raw) => {
                    components.push(Box::new(Into::<offline_data::Light>::into(raw)));
                }
                raw_data::Component::Physics(raw) => {
                    components.push(Box::new(Into::<offline_data::Physics>::into(raw)));
                }
            }
        }
        Self {
            name: raw.name,
            children,
            parent,
            components,
        }
    }
}

impl From<raw_data::Transform> for offline_data::Transform {
    fn from(raw: raw_data::Transform) -> Self {
        Self {
            position: raw.position,
            rotation: raw.rotation,
            scale: raw.scale,
            apply_to_children: raw.apply_to_children,
        }
    }
}

impl From<raw_data::Visual> for offline_data::Visual {
    fn from(raw: raw_data::Visual) -> Self {
        Self {
            renderable_geometry: raw.renderable_geometry,
            shadow_receiver: raw.shadow_receiver,
            shadow_caster_sun: raw.shadow_caster_sun,
            shadow_caster_local: raw.shadow_caster_local,
            gi_contribution: raw.gi_contribution.into(),
        }
    }
}

impl From<raw_data::GIContribution> for offline_data::GIContribution {
    fn from(raw: raw_data::GIContribution) -> Self {
        match raw {
            raw_data::GIContribution::Default => Self::Default,
            raw_data::GIContribution::Blocker => Self::Blocker,
            raw_data::GIContribution::Exclude => Self::Exclude,
        }
    }
}

impl From<raw_data::GlobalIllumination> for offline_data::GlobalIllumination {
    fn from(_raw: raw_data::GlobalIllumination) -> Self {
        Self {}
    }
}

impl From<raw_data::NavMesh> for offline_data::NavMesh {
    fn from(raw: raw_data::NavMesh) -> Self {
        Self {
            voxelisation_config: raw.voxelisation_config.into(),
            layer_config: raw
                .layer_config
                .iter()
                .map(Into::<offline_data::NavMeshLayerConfig>::into)
                .collect(),
        }
    }
}

impl From<raw_data::VoxelisationConfig> for offline_data::VoxelisationConfig {
    fn from(_raw: raw_data::VoxelisationConfig) -> Self {
        Self {}
    }
}

impl From<&raw_data::NavMeshLayerConfig> for offline_data::NavMeshLayerConfig {
    fn from(_raw: &raw_data::NavMeshLayerConfig) -> Self {
        Self {}
    }
}

impl From<raw_data::View> for offline_data::View {
    fn from(raw: raw_data::View) -> Self {
        Self {
            fov: raw.fov,
            near: raw.near,
            far: raw.far,
            projection_type: raw.projection_type.into(),
        }
    }
}

impl From<raw_data::ProjectionType> for offline_data::ProjectionType {
    fn from(raw: raw_data::ProjectionType) -> Self {
        match raw {
            raw_data::ProjectionType::Orthogonal => Self::Orthogonal,
            raw_data::ProjectionType::Perspective => Self::Perspective,
        }
    }
}

impl From<raw_data::Light> for offline_data::Light {
    fn from(_raw: raw_data::Light) -> Self {
        Self {}
    }
}

impl From<raw_data::Physics> for offline_data::Physics {
    fn from(raw: raw_data::Physics) -> Self {
        Self {
            dynamic: raw.dynamic,
            collision_geometry: raw.collision_geometry,
        }
    }
}

// ----- Instance conversions -----

impl FromRaw<raw_data::Instance> for offline_data::Instance {
    fn from_raw(
        raw: raw_data::Instance,
        references: &HashMap<ResourcePathName, ResourceId>,
    ) -> Self {
        Self {
            original: lookup_reference(references, &raw.original),
        }
    }
}

// ----- Material conversions -----

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

// ----- Mesh conversions -----

impl FromRaw<raw_data::Mesh> for offline_data::Mesh {
    fn from_raw(raw: raw_data::Mesh, references: &HashMap<ResourcePathName, ResourceId>) -> Self {
        Self {
            sub_meshes: raw
                .sub_meshes
                .iter()
                .map(|sub_mesh| offline_data::SubMesh::from_raw(sub_mesh, references))
                .collect(),
        }
    }
}

impl FromRaw<&raw_data::SubMesh> for offline_data::SubMesh {
    fn from_raw(
        raw: &raw_data::SubMesh,
        references: &HashMap<ResourcePathName, ResourceId>,
    ) -> Self {
        Self {
            positions: raw.positions.clone(),
            normals: raw.normals.clone(),
            uvs: raw.uvs.clone(),
            indices: raw.indices.clone(),
            material: lookup_reference(references, &raw.material).unwrap(),
        }
    }
}
