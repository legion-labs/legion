use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use lgn_core::Name;
use lgn_data_runtime::prelude::*;
use lgn_transform::prelude::{GlobalTransform, Transform};

use crate::scene_instance::SceneInstance;
use lgn_data_runtime::Component as RuntimeComponent;
//use lgn_ecs::prelude::Component as BevyComponent;
use lgn_ecs::{prelude::Commands, system::EntityCommands};
use sample_data::runtime::Entity as RuntimeEntity;

struct Inner {
    active_scenes: HashMap<ResourceTypeAndId, SceneInstance>,
    pending: Vec<Handle<RuntimeEntity>>,
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

    pub(crate) fn add_pending(&self, entity: Handle<RuntimeEntity>) {
        self.inner.write().unwrap().pending.push(entity);
    }
}

#[async_trait]
impl ComponentInstaller for SceneManager {
    /// Consume a resource return the installed version
    fn install_component(
        &self,
        component: &dyn RuntimeComponent,
        entity_command: &mut EntityCommands<'_, '_, '_>,
    ) -> Result<(), AssetRegistryError> {
        // Assign Name
        if let Some(name) = component.downcast_ref::<sample_data::runtime::Name>() {
            entity_command.insert(Name::new(name.name.clone()));
        }

        // Assign Local and Global Transform
        if let Some(local_transform) = component.downcast_ref::<sample_data::runtime::Transform>() {
            entity_command.insert(Transform {
                translation: local_transform.position,
                rotation: local_transform.rotation,
                scale: local_transform.scale,
            });
            entity_command.insert(GlobalTransform::identity());
        }
        Ok(())
    }
}
