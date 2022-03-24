use std::collections::{BTreeMap, HashSet};

use lgn_app::App;
use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::{Entity, Query, ResMut, Without};
use lgn_tracing::span_fn;

use crate::{
    components::{ManipulatorComponent, MaterialComponent, VisualComponent},
    labels::RenderStage,
};

#[derive(Default)]
pub(crate) struct MissingVisualTracker {
    model_to_entity_set: BTreeMap<ResourceTypeAndId, HashSet<Entity>>,
    models_added: Vec<ResourceTypeAndId>,
}

impl MissingVisualTracker {
    pub fn init_ecs(app: &mut App) {
        app.add_system_to_stage(RenderStage::Prepare, update_missing_visuals);
    }

    pub(crate) fn add_model_entity_dependency(
        &mut self,
        model_resource_id: ResourceTypeAndId,
        entity: Entity,
    ) {
        if let Some(entry) = self.model_to_entity_set.get_mut(&model_resource_id) {
            entry.insert(entity);
        } else {
            let mut set = HashSet::new();
            set.insert(entity);
            self.model_to_entity_set.insert(model_resource_id, set);
        }
    }

    pub(crate) fn add_changed_model(&mut self, resource_id: ResourceTypeAndId) {
        self.models_added.push(resource_id);
    }

    pub(crate) fn get_entities_to_update(&mut self) -> HashSet<Entity> {
        let mut entities = HashSet::new();
        for model_resource_id in &self.models_added {
            if let Some(entry) = self.model_to_entity_set.get(model_resource_id) {
                for entity in entry {
                    entities.insert(*entity);
                }
                self.model_to_entity_set.remove_entry(model_resource_id);
            }
        }
        self.models_added.clear();
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
