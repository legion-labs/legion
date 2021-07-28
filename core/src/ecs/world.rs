use std::{cell::RefCell, rc::Weak};

use super::entity::{Entity, EntityIdentifier, EntityIdentifierGenerator, INVALID_ENTITY_ID};
use super::ids::IdentifierGenerator;

pub struct World {
    id: WorldIdentifier,
    entities: Vec<Entity>,
    entity_id_generator: Weak<RefCell<EntityIdentifierGenerator>>,
}

pub type WorldIdentifier = u16;
//const INVALID_WORLD_ID: WorldIdentifier = WorldIdentifier::MAX;

pub type WorldIdentifierGenerator = IdentifierGenerator<WorldIdentifier>;

impl World {
    pub fn new(
        id: WorldIdentifier,
        entity_id_generator: Weak<RefCell<EntityIdentifierGenerator>>,
    ) -> Self {
        Self {
            id,
            entities: Vec::new(),
            entity_id_generator,
        }
    }

    pub fn create_entity(&mut self) -> EntityIdentifier {
        if let Some(id_generator) = self.entity_id_generator.upgrade() {
            let id = (*id_generator).borrow_mut().get_new_id();
            self.entities.push(Entity::from(id));
            return id;
        }
        INVALID_ENTITY_ID
    }

    pub fn get_id(&self) -> WorldIdentifier {
        self.id
    }

    pub fn get_entities(&self) -> &Vec<Entity> {
        &self.entities
    }
}
