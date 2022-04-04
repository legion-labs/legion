use std::collections::HashMap;

use lgn_data_offline::{resource::ResourcePathName, ResourcePathId};
use lgn_data_runtime::{Component, Resource, ResourceType, ResourceTypeAndId};
use lgn_tracing::{error, info};
use sample_data::offline as offline_data;

use super::raw_data;

pub trait FromRaw<T> {
    fn from_raw(raw: T, references: &HashMap<ResourcePathName, ResourceTypeAndId>) -> Self;
}

fn lookup_reference(
    references: &HashMap<ResourcePathName, ResourceTypeAndId>,
    path: &str,
) -> Option<ResourceTypeAndId> {
    let path = ResourcePathName::new(path);
    references.get(&path).copied()
}

fn source_resource(path: &str) -> &str {
    if let (Some(l), Some(r)) = { (path.find('('), path.rfind(')')) } {
        return source_resource(&path[(l + 1)..r]);
    }
    path.find(',').map_or(path, |sep| &path[..sep])
}

fn push_transforms(mut id: ResourcePathId, path: &str) -> ResourcePathId {
    let mut l = path.rfind('(').unwrap_or(0);
    let r = l + {
        let path = &path[l..];
        let s = path.find(',').unwrap_or(path.len());
        let b = path.find(')').unwrap_or(path.len());
        s.min(b)
    };

    let mut left = &path[..l];
    let mut right = &path[r..];

    loop {
        if left.is_empty() {
            break;
        }

        l = left.rfind('(').unwrap_or(0);
        let asset_type = left[l..].trim_start_matches('(');

        let name = {
            let r = right.find(')').unwrap_or(right.len() - 1);
            let name = &right[..r];
            right = &right[r + 1..];

            if let (Some(l), Some(r)) = (name.find('\''), name.rfind('\'')) {
                if l == r {
                    None
                } else {
                    Some(&name[l + 1..r])
                }
            } else {
                None
            }
        };

        let kind = ResourceType::new(asset_type.as_bytes());
        if let Some(name) = name {
            id = id.push_named(kind, name);
        } else {
            id = id.push(kind);
        }

        left = &left[0..l];
    }

    id
}

// Resolved a raw representation of asset_path such as
// "runtime_texture(offline_texture(image/ground.psd, 'albedo'))" into a offline
// ResourcePathId such as
// "Some((13b5a84e,000000007f8d831386fd3fef)|1960578643_albedo)"
fn lookup_asset_path(
    references: &HashMap<ResourcePathName, ResourceTypeAndId>,
    path: &str,
) -> Option<ResourcePathId> {
    let source = source_resource(path);
    let output = lookup_reference(references, source)
        .map(ResourcePathId::from)
        .map(|id| push_transforms(id, path));
    match &output {
        Some(resolved_path) => {
            info!("Path Resolved: {} -> {}", path, resolved_path);
        }
        None => {
            error!("Failed to resolve path: {}", path);
        }
    }
    output
}

// ----- Entity conversions -----

impl FromRaw<raw_data::Entity> for offline_data::Entity {
    fn from_raw(
        raw: raw_data::Entity,
        references: &HashMap<ResourcePathName, ResourceTypeAndId>,
    ) -> Self {
        let children: Vec<ResourcePathId> = raw
            .children
            .iter()
            .filter_map(|path| lookup_asset_path(references, path))
            .collect();
        let parent = match raw.parent {
            Some(parent) => lookup_asset_path(references, &parent)
                .map(|res| res.push(sample_data::runtime::Entity::TYPE)),
            None => None,
        };
        let mut components: Vec<Box<dyn Component>> =
            vec![Box::new(offline_data::Name { name: raw.name })];
        for component in raw.components {
            match component {
                raw_data::Component::Transform(raw) => {
                    components.push(Box::new(Into::<offline_data::Transform>::into(raw)));
                }
                raw_data::Component::Visual(raw) => {
                    components.push(Box::new(offline_data::Visual::from_raw(raw, references)));
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
                raw_data::Component::GltfLoader(raw) => {
                    components.push(Box::new(offline_data::GltfLoader::from_raw(
                        raw, references,
                    )));
                }
            }
        }

        Self {
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
        }
    }
}

impl FromRaw<raw_data::Visual> for offline_data::Visual {
    fn from_raw(
        raw: raw_data::Visual,
        references: &HashMap<ResourcePathName, ResourceTypeAndId>,
    ) -> Self {
        Self {
            renderable_geometry: if let Some(path) = &raw.renderable_geometry {
                lookup_asset_path(references, path)
            } else {
                None
            },
            color: lgn_graphics_data::Color::from((
                raw.color.x as u8,
                raw.color.y as u8,
                raw.color.z as u8,
            )),
            color_blend: raw.color_blend,
            shadow_receiver: raw.shadow_receiver,
            shadow_caster_sun: raw.shadow_caster_sun,
            shadow_caster_local: raw.shadow_caster_local,
            gi_contribution: raw.gi_contribution.into(),
        }
    }
}

impl From<raw_data::GIContribution> for sample_data::GIContribution {
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
    fn from(_raw: raw_data::NavMesh) -> Self {
        Self {
           /* voxelisation_config: raw.voxelisation_config.into(),
            layer_config: raw
                .layer_config
                .iter()
                .map(Into::<offline_data::NavMeshLayerConfig>::into)
                .collect(),*/
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

impl From<raw_data::ProjectionType> for sample_data::ProjectionType {
    fn from(raw: raw_data::ProjectionType) -> Self {
        match raw {
            raw_data::ProjectionType::Orthogonal => Self::Orthogonal,
            raw_data::ProjectionType::Perspective => Self::Perspective,
        }
    }
}

impl From<raw_data::Light> for offline_data::Light {
    fn from(raw: raw_data::Light) -> Self {
        Self {
            light_type: raw.light_type,
            color: lgn_graphics_data::Color::from((
                raw.color.x as u8,
                raw.color.y as u8,
                raw.color.z as u8,
            )),
            radiance: raw.radiance,
            enabled: raw.enabled,
            cone_angle: raw.cone_angle,
        }
    }
}

impl FromRaw<raw_data::GltfLoader> for offline_data::GltfLoader {
    fn from_raw(
        raw: raw_data::GltfLoader,
        references: &HashMap<ResourcePathName, ResourceTypeAndId>,
    ) -> Self {
        Self {
            models: raw
                .models
                .iter()
                .filter_map(|model| lookup_asset_path(references, model))
                .collect(),

            materials: raw
                .materials
                .iter()
                .filter_map(|material| lookup_asset_path(references, material))
                .collect(),

            textures: raw
                .textures
                .iter()
                .filter_map(|texture| lookup_asset_path(references, texture))
                .collect(),
        }
    }
}

// ----- Instance conversions -----

impl FromRaw<raw_data::Instance> for offline_data::Instance {
    fn from_raw(
        raw: raw_data::Instance,
        references: &HashMap<ResourcePathName, ResourceTypeAndId>,
    ) -> Self {
        Self {
            original: lookup_asset_path(references, &raw.original),
        }
    }
}

// ----- Material conversions -----

impl FromRaw<raw_data::Material> for lgn_graphics_data::offline::Material {
    fn from_raw(
        raw: raw_data::Material,
        references: &HashMap<ResourcePathName, ResourceTypeAndId>,
    ) -> Self {
        Self {
            albedo: lookup_asset_path(references, &raw.albedo),
            normal: lookup_asset_path(references, &raw.normal),
            roughness: lookup_asset_path(references, &raw.roughness),
            metalness: lookup_asset_path(references, &raw.metalness),
            base_albedo: raw.base_albedo,
            base_metalness: raw.base_metalness,
            base_roughness: raw.base_roughness,
            reflectance: raw.reflectance,
        }
    }
}
