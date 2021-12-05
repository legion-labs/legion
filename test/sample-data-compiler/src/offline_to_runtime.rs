use std::any::Any;

use legion_data_offline::ResourcePathId;
use legion_data_runtime::{Reference, Resource};
use sample_data_offline as offline_data;
use sample_data_runtime as runtime_data;

pub fn find_derived_path(path: &ResourcePathId) -> ResourcePathId {
    let offline_type = path.content_type();
    match offline_type {
        offline_data::Entity::TYPE => path.push(runtime_data::Entity::TYPE),
        offline_data::Instance::TYPE => path.push(runtime_data::Instance::TYPE),
        offline_data::Mesh::TYPE => path.push(runtime_data::Mesh::TYPE),
        legion_graphics_offline::PsdFile::TYPE => path
            .push(legion_graphics_offline::Texture::TYPE)
            .push(legion_graphics_runtime::Texture::TYPE),
        legion_graphics_offline::Material::TYPE => {
            path.push(legion_graphics_runtime::Material::TYPE)
        }
        generic_data_offline::DebugCube::TYPE => path.push(generic_data_runtime::DebugCube::TYPE),
        _ => {
            panic!("unrecognized offline type {}", offline_type);
        }
    }
}

pub fn to_reference<T>(path: &Option<ResourcePathId>) -> Option<Reference<T>>
where
    T: Any + Resource,
{
    path.as_ref()
        .map(|path| Reference::Passive(path.resource_id()))
}

pub trait FromOffline<T> {
    fn from_offline(offline: &T) -> Self;
}

// ----- Entity conversions -----

impl FromOffline<offline_data::Entity> for runtime_data::Entity {
    fn from_offline(offline: &offline_data::Entity) -> Self {
        let children = offline
            .children
            .iter()
            .map(|path| Reference::Passive(path.resource_id()))
            .collect();
        let mut components: Vec<Box<dyn runtime_data::Component>> = Vec::new();
        for component in &offline.components {
            if let Some(transform) = component.downcast_ref::<offline_data::Transform>() {
                components.push(Box::new(runtime_data::Transform::from_offline(transform)));
            } else if let Some(visual) = component.downcast_ref::<offline_data::Visual>() {
                components.push(Box::new(runtime_data::Visual::from_offline(visual)));
            } else if let Some(gi) = component.downcast_ref::<offline_data::GlobalIllumination>() {
                components.push(Box::new(runtime_data::GlobalIllumination::from_offline(gi)));
            } else if let Some(nav_mesh) = component.downcast_ref::<offline_data::NavMesh>() {
                components.push(Box::new(runtime_data::NavMesh::from_offline(nav_mesh)));
            } else if let Some(view) = component.downcast_ref::<offline_data::View>() {
                components.push(Box::new(runtime_data::View::from_offline(view)));
            } else if let Some(light) = component.downcast_ref::<offline_data::Light>() {
                components.push(Box::new(runtime_data::Light::from_offline(light)));
            } else if let Some(physics) = component.downcast_ref::<offline_data::Physics>() {
                components.push(Box::new(runtime_data::Physics::from_offline(physics)));
            }
        }
        Self {
            name: offline.name.clone(),
            children,
            parent: to_reference(&offline.parent),
            components,
        }
    }
}

impl FromOffline<offline_data::Transform> for runtime_data::Transform {
    fn from_offline(offline: &offline_data::Transform) -> Self {
        Self {
            position: offline.position,
            rotation: offline.rotation,
            scale: offline.scale,
            apply_to_children: offline.apply_to_children,
        }
    }
}

impl FromOffline<offline_data::Visual> for runtime_data::Visual {
    fn from_offline(offline: &offline_data::Visual) -> Self {
        Self {
            renderable_geometry: to_reference(&offline.renderable_geometry),
            shadow_receiver: offline.shadow_receiver,
            shadow_caster_sun: offline.shadow_caster_sun,
            shadow_caster_local: offline.shadow_caster_local,
            gi_contribution: runtime_data::GIContribution::from_offline(&offline.gi_contribution),
        }
    }
}

impl FromOffline<offline_data::GIContribution> for runtime_data::GIContribution {
    fn from_offline(offline: &offline_data::GIContribution) -> Self {
        match offline {
            offline_data::GIContribution::Default => Self::Default,
            offline_data::GIContribution::Blocker => Self::Blocker,
            offline_data::GIContribution::Exclude => Self::Exclude,
        }
    }
}

impl FromOffline<offline_data::GlobalIllumination> for runtime_data::GlobalIllumination {
    fn from_offline(_offline: &offline_data::GlobalIllumination) -> Self {
        Self {}
    }
}

impl FromOffline<offline_data::NavMesh> for runtime_data::NavMesh {
    fn from_offline(offline: &offline_data::NavMesh) -> Self {
        Self {
            voxelisation_config: runtime_data::VoxelisationConfig::from_offline(
                &offline.voxelisation_config,
            ),
            layer_config: offline
                .layer_config
                .iter()
                .map(runtime_data::NavMeshLayerConfig::from_offline)
                .collect(),
        }
    }
}

impl FromOffline<offline_data::VoxelisationConfig> for runtime_data::VoxelisationConfig {
    fn from_offline(_offline: &offline_data::VoxelisationConfig) -> Self {
        Self {}
    }
}

impl FromOffline<offline_data::NavMeshLayerConfig> for runtime_data::NavMeshLayerConfig {
    fn from_offline(_offline: &offline_data::NavMeshLayerConfig) -> Self {
        Self {}
    }
}

impl FromOffline<offline_data::View> for runtime_data::View {
    fn from_offline(offline: &offline_data::View) -> Self {
        Self {
            fov: offline.fov,
            near: offline.near,
            far: offline.far,
            projection_type: runtime_data::ProjectionType::from_offline(&offline.projection_type),
        }
    }
}

impl FromOffline<offline_data::ProjectionType> for runtime_data::ProjectionType {
    fn from_offline(offline: &offline_data::ProjectionType) -> Self {
        match *offline {
            offline_data::ProjectionType::Orthogonal => Self::Orthogonal,
            offline_data::ProjectionType::Perspective => Self::Perspective,
        }
    }
}

impl FromOffline<offline_data::Light> for runtime_data::Light {
    fn from_offline(_offline: &offline_data::Light) -> Self {
        Self {}
    }
}

impl FromOffline<offline_data::Physics> for runtime_data::Physics {
    fn from_offline(offline: &offline_data::Physics) -> Self {
        Self {
            dynamic: offline.dynamic,
            collision_geometry: to_reference(&offline.collision_geometry),
        }
    }
}

// ----- Instance conversions -----

impl FromOffline<offline_data::Instance> for runtime_data::Instance {
    fn from_offline(offline: &offline_data::Instance) -> Self {
        Self {
            original: to_reference(&offline.original),
        }
    }
}

// ----- Mesh conversions -----

impl FromOffline<offline_data::Mesh> for runtime_data::Mesh {
    fn from_offline(offline: &offline_data::Mesh) -> Self {
        Self {
            sub_meshes: offline
                .sub_meshes
                .iter()
                .map(runtime_data::SubMesh::from_offline)
                .collect(),
        }
    }
}

impl FromOffline<offline_data::SubMesh> for runtime_data::SubMesh {
    fn from_offline(offline: &offline_data::SubMesh) -> Self {
        Self {
            positions: offline.positions.clone(),
            normals: offline.normals.clone(),
            uvs: offline.uvs.clone(),
            indices: offline.indices.clone(),
            material: to_reference(&offline.material),
        }
    }
}
