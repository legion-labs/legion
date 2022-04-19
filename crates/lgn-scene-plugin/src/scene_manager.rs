use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use lgn_data_runtime::prelude::*;

use crate::scene_instance::SceneInstance;
use lgn_ecs::prelude::Commands;

struct Inner {
    active_scenes: HashMap<ResourceTypeAndId, SceneInstance>,
    pending: Vec<Handle<sample_data::runtime::Entity>>,
}

pub struct SceneManager {
    inner: RwLock<Inner>,
}

impl SceneManager {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: RwLock::new(Inner {
                active_scenes: HashMap::new(),
                pending: Vec::new(),
            }),
        })
    }

    pub(crate) fn update(&self, asset_registry: &AssetRegistry, commands: &mut Commands<'_, '_>) {
        let pending = std::mem::take(&mut self.inner.write().unwrap().pending);

        for handle in pending {
            let root_id = handle.id();

            let mut guard = self.inner.write().unwrap();
            let scene = guard
                .active_scenes
                .entry(root_id)
                .or_insert_with(|| SceneInstance::new(root_id, handle.clone()));
            scene.spawn_entity_hierarchy(handle, asset_registry, commands);
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

    pub(crate) fn add_pending(&self, entity: Handle<sample_data::runtime::Entity>) {
        self.inner.write().unwrap().pending.push(entity);
    }
}
