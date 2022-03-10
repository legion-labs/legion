use lgn_ecs::prelude::Entity;
use lgn_transform::components::Transform;

#[derive(Debug)]
pub enum PickingEvent {
    ClearSelection,
    EntityPicked(Entity),
    ApplyTransaction(Entity, Transform),
}
