use std::collections::{BTreeMap, HashSet};

use lgn_app::App;
use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::{
    prelude::{Entity, IntoExclusiveSystem, Query, Res},
    schedule::ExclusiveSystemDescriptorCoercion,
};

use crate::{components::VisualComponent, labels::RenderStage, Renderer};

#[derive(Default)]
pub(crate) struct MissingVisualTracker {
    resource_to_entity_set: BTreeMap<ResourceTypeAndId, HashSet<Entity>>,
    resource_added: HashSet<ResourceTypeAndId>,
}

impl MissingVisualTracker {
    pub fn init_ecs(app: &mut App) {
        app.add_system_to_stage(
            RenderStage::Prepare,
            update_missing_visuals.exclusive_system().at_start(),
        );
    }

    pub(crate) fn add_resource_entity_dependency(
        &mut self,
        resource_id: ResourceTypeAndId,
        entity: Entity,
    ) {
        if let Some(entry) = self.resource_to_entity_set.get_mut(&resource_id) {
            entry.insert(entity);
        } else {
            let mut set = HashSet::new();
            set.insert(entity);
            self.resource_to_entity_set.insert(resource_id, set);
        }
    }

    pub(crate) fn add_changed_resource(&mut self, resource_id: ResourceTypeAndId) {
        self.resource_added.insert(resource_id);
    }

    fn get_entities_to_update(&mut self) -> HashSet<Entity> {
        let mut entities = HashSet::new();
        for resource_id in &self.resource_added {
            if let Some(entry) = self.resource_to_entity_set.get(resource_id) {
                for entity in entry {
                    entities.insert(*entity);
                }
                self.resource_to_entity_set.remove_entry(resource_id);
            }
        }
        self.resource_added.clear();
        entities
    }
}

#[allow(clippy::needless_pass_by_value)]
fn update_missing_visuals(
    renderer: Res<'_, Renderer>,
    mut visuals_query: Query<'_, '_, (Entity, &mut VisualComponent)>,
) {
    let mut missing_visuals_tracker = renderer
        .render_resources()
        .get_mut::<MissingVisualTracker>();
    for entity in missing_visuals_tracker.get_entities_to_update() {
        if let Ok((_entity, mut visual_component)) = visuals_query.get_mut(entity) {
            visual_component.as_mut(); // Will trigger 'changed' to the visual component and it will be updated on the next update_gpu_instances()
        }
    }
}
