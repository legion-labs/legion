use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use lgn_data_runtime::prelude::*;

use crate::scene_instance::SceneInstance;
use lgn_ecs::prelude::Commands;

struct Inner {
    active_scenes: HashMap<ResourceTypeAndId, SceneInstance>,
    pending_scene: Vec<Handle<sample_data::runtime::Entity>>,
    pending_reload: Vec<Handle<sample_data::runtime::Entity>>,
}

pub struct SceneManager {
    inner: RwLock<Inner>,
}

impl SceneManager {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: RwLock::new(Inner {
                active_scenes: HashMap::new(),
                pending_scene: Vec::new(),
                pending_reload: Vec::new(),
            }),
        })
    }

    pub(crate) fn update(&self, asset_registry: &AssetRegistry, commands: &mut Commands<'_, '_>) {
        let (pending_scene, pending_reload) = {
            let mut guard = self.inner.write().unwrap();
            let pending_scene = std::mem::take(&mut guard.pending_scene);
            let pending_reload = std::mem::take(&mut guard.pending_reload);
            (pending_scene, pending_reload)
        };

        for handle in pending_scene {
            let root_id = handle.id();
            let mut guard = self.inner.write().unwrap();
            let scene = guard
                .active_scenes
                .entry(root_id)
                .or_insert_with(|| SceneInstance::new(root_id, handle.clone()));
            scene.spawn_entity_hierarchy(handle, asset_registry, commands);
        }

        for handle in pending_reload {
            let resource_id = handle.id();
            for scene_instance in self.inner.write().unwrap().active_scenes.values_mut() {
                if let Some(_entity) = scene_instance.find_entity(&resource_id) {
                    scene_instance.spawn_entity_hierarchy(handle.clone(), asset_registry, commands);
                }
            }
        }
    }

    pub(crate) fn close_scene(&self, root_id: &ResourceTypeAndId, commands: &mut Commands<'_, '_>) {
        if let Some(mut scene) = self.inner.write().unwrap().active_scenes.remove(root_id) {
            scene.unspawn_all(commands);
        }
    }

    /// Return all the Bevy entities for a given `ResourceTypeAndId`
    pub fn find_entities(
        &self,
        resource_id: &ResourceTypeAndId,
    ) -> Option<Vec<lgn_ecs::prelude::Entity>> {
        let mut result: Option<Vec<lgn_ecs::prelude::Entity>> = None;
        for scene_instance in self.inner.read().unwrap().active_scenes.values() {
            if let Some(entity) = scene_instance.find_entity(resource_id) {
                result.get_or_insert(Vec::new()).push(*entity);
            }
        }
        result
    }

    pub async fn notify_changed_resources(
        &self,
        changed: &[ResourceTypeAndId],
        asset_registry: &Arc<AssetRegistry>,
    ) {
        let mut reloads = Vec::new();
        for scene_instance in self.inner.write().unwrap().active_scenes.values_mut() {
            reloads.extend(scene_instance.notify_changed_resources(changed, asset_registry));
        }

        for (resource_id, job_result) in reloads {
            match job_result.await {
                Ok(load_result) => match load_result {
                    Ok(handle) => {
                        self.inner
                            .write()
                            .unwrap()
                            .pending_reload
                            .push(handle.into());
                    }
                    Err(load_err) => {
                        lgn_tracing::error!("Failed to reload {} {:?}", resource_id, load_err);
                    }
                },
                Err(job_error) => lgn_tracing::error!("{}", job_error),
            }
        }
    }

    pub(crate) fn add_pending_scene(&self, entity: Handle<sample_data::runtime::Entity>) {
        self.inner.write().unwrap().pending_scene.push(entity);
    }
}
