use super::entity::EntityIdentifierGenerator;
use super::world::{World, WorldIdentifier, WorldIdentifierGenerator};
use std::{cell::RefCell, rc::Rc};

pub struct Project {
    world_id_generator: WorldIdentifierGenerator,
    worlds: Vec<World>,
    entity_id_generator: Rc<RefCell<EntityIdentifierGenerator>>,
}

impl Project {
    // World management

    pub fn create_world(&mut self) -> WorldIdentifier {
        let id = self.world_id_generator.get_new_id();
        let world = World::new(id, Rc::downgrade(&self.entity_id_generator));
        self.worlds.push(world);
        id
    }

    pub fn get_world(&self, id: WorldIdentifier) -> Option<&World> {
        if let Some(world) = self.worlds.iter().find(|world| world.get_id() == id) {
            Some(world)
        } else {
            None
        }
    }

    pub fn get_world_mut(&mut self, id: WorldIdentifier) -> Option<&mut World> {
        if let Some(world) = self.worlds.iter_mut().find(|world| world.get_id() == id) {
            Some(world)
        } else {
            None
        }
    }

    pub fn get_worlds(&self) -> &Vec<World> {
        &self.worlds
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            world_id_generator: WorldIdentifierGenerator::default(),
            worlds: Vec::new(),
            entity_id_generator: Rc::new(RefCell::new(EntityIdentifierGenerator::default())),
        }
    }
}
