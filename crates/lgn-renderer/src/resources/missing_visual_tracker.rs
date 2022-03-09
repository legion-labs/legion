use std::collections::{BTreeMap, HashSet};

use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::{Entity, Query, ResMut, Without};
use lgn_tracing::span_fn;

use crate::components::{ManipulatorComponent, MaterialComponent, VisualComponent};

#[derive(Default)]
pub(crate) struct MissingVisualTracker {
    entities: BTreeMap<ResourceTypeAndId, HashSet<Entity>>,
    visuals_added: Vec<ResourceTypeAndId>,
}

impl MissingVisualTracker {
    pub(crate) fn add_entity(&mut self, resource_id: ResourceTypeAndId, entity_id: Entity) {
        if let Some(entry) = self.entities.get_mut(&resource_id) {
            entry.insert(entity_id);
        } else {
            let mut set = HashSet::new();
            set.insert(entity_id);
            self.entities.insert(resource_id, set);
        }
    }

    pub(crate) fn add_visuals(&mut self, resource_id: ResourceTypeAndId) {
        self.visuals_added.push(resource_id);
    }

    pub(crate) fn get_entities_to_update(&mut self) -> HashSet<Entity> {
        let mut entities = HashSet::new();
        for visual in &self.visuals_added {
            if let Some(entry) = self.entities.get(visual) {
                for entity in entry {
                    entities.insert(*entity);
                }
                self.entities.remove_entry(visual);
            }
        }
        self.visuals_added.clear();
        entities
    }
}

#[span_fn]
#[allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_arguments
)]
pub(crate) fn update_missing_visuals(
    mut missing_visuals_tracker: ResMut<'_, MissingVisualTracker>,
    mut visuals_query: Query<
        '_,
        '_,
        (Entity, &mut VisualComponent, Option<&MaterialComponent>),
        Without<ManipulatorComponent>,
    >,
) {
    for entity in missing_visuals_tracker.get_entities_to_update() {
        if let Ok((_entity, mut visual_component, _mat_component)) = visuals_query.get_mut(entity) {
            visual_component.as_mut(); // Will trigger 'changed' to the visual component and it will be updated on the next update_gpu_instances()
        }
    }
}
