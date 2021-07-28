#[cfg(test)]
use crate::prelude::*;

pub mod entity;
pub mod ids;
pub mod project;
pub mod reflection;
pub mod system;
pub mod world;

pub use entity::{Component, Entity};
pub use project::Project;
pub use system::System;
pub use world::World;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_entities() {
        let mut project = Project::default();

        let world = project.create_world();
        if let Some(world) = project.get_world_mut(world) {
            let entity_1 = world.create_entity();
            assert!(world.get_entities().len() == 1);

            let entity_2 = world.create_entity();
            assert!(entity_1 != entity_2);
            assert!(world.get_entities().len() == 2);
        }
        assert!(project.get_worlds().len() == 1);

        let world = project.create_world();
        if let Some(world) = project.get_world_mut(world) {
            let _entity = world.create_entity();
            assert!(world.get_entities().len() == 1);
        }
        assert!(project.get_worlds().len() == 2);
    }

    pub struct Position(Vector3);
    impl Component for Position {}

    pub struct Velocity(Vector3);
    impl Component for Velocity {}
}
