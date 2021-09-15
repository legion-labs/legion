use legion_data_runtime::AssetId;
use sample_data_compiler::{
    offline_data::{self, Transform},
    runtime_data,
};

pub trait FromOffline<T> {
    fn from_offline(offline: &T) -> Self;
}

// ----- Entity conversions -----

impl FromOffline<offline_data::Entity> for runtime_data::Entity {
    fn from_offline(offline: &offline_data::Entity) -> Self {
        let children: Vec<AssetId> = Vec::new();
        let mut components: Vec<Box<dyn runtime_data::Component>> = Vec::new();
        for component in &offline.components {
            if let Some(transform) = component.downcast_ref::<Transform>() {
                components.push(Box::new(runtime_data::Transform::from_offline(transform)));
            }
        }
        Self {
            name: offline.name.clone(),
            children,
            parent: None,
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

// ----- Instance conversions -----

impl FromOffline<offline_data::Instance> for runtime_data::Instance {
    fn from_offline(_offline: &offline_data::Instance) -> Self {
        Self {}
    }
}

// ----- Material conversions -----

impl FromOffline<offline_data::Material> for runtime_data::Material {
    fn from_offline(offline: &offline_data::Material) -> Self {
        Self {
            albedo: offline.albedo.clone(),
            normal: offline.normal.clone(),
            roughness: offline.roughness.clone(),
            metalness: offline.metalness.clone(),
        }
    }
}

// ----- Mesh conversions -----

impl FromOffline<offline_data::Mesh> for runtime_data::Mesh {
    fn from_offline(_offline: &offline_data::Mesh) -> Self {
        Self {}
    }
}
