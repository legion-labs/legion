use std::cell::RefCell;
use std::ops::AddAssign;
use std::rc::{Rc, Weak};

pub struct Entity {
    id: EntityIdentifier,
}

pub trait Component {}

pub struct World {
    id: WorldIdentifier,
    name: String,
    entities: Vec<Entity>,
    entity_id_generator: Weak<RefCell<EntityIdentifierGenerator>>,
}

impl World {
    pub fn new(
        id: WorldIdentifier,
        name: String,
        entity_id_generator: Weak<RefCell<EntityIdentifierGenerator>>,
    ) -> Self {
        Self {
            id,
            name,
            entities: Vec::new(),
            entity_id_generator,
        }
    }

    fn create_entity(&mut self) -> EntityIdentifier {
        if let Some(id_generator) = self.entity_id_generator.upgrade() {
            let id = (*id_generator).borrow_mut().get_new_id();
            self.entities.push(Entity { id });
            return id;
        }
        INVALID_ENTITY_ID
    }
}
pub trait One {
    fn one() -> Self;
}

pub type EntityIdentifier = u64;
const INVALID_ENTITY_ID: EntityIdentifier = EntityIdentifier::MAX;

impl One for EntityIdentifier {
    fn one() -> Self {
        1
    }
}

pub type WorldIdentifier = u16;
//const INVALID_WORLD_ID: WorldIdentifier = WorldIdentifier::MAX;

impl One for WorldIdentifier {
    fn one() -> Self {
        1
    }
}

pub struct IdentifierGenerator<T>
where
    T: AddAssign + Copy + Default + One,
{
    next_valid_id: T,
}

impl<T> IdentifierGenerator<T>
where
    T: AddAssign + Copy + Default + One,
{
    fn new() -> Self {
        Self {
            next_valid_id: T::default(),
        }
    }

    fn get_new_id(&mut self) -> T {
        self.next_valid_id += T::one();
        self.next_valid_id
    }
}

type EntityIdentifierGenerator = IdentifierGenerator<EntityIdentifier>;
type WorldIdentifierGenerator = IdentifierGenerator<WorldIdentifier>;

pub struct Project {
    name: String,
    world_id_generator: WorldIdentifierGenerator,
    worlds: Vec<World>,
    entity_id_generator: Rc<RefCell<EntityIdentifierGenerator>>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            name,
            world_id_generator: WorldIdentifierGenerator::new(),
            worlds: Vec::new(),
            entity_id_generator: Rc::new(RefCell::new(EntityIdentifierGenerator::new())),
        }
    }

    // World management

    pub fn create_world(&mut self, name: String) -> WorldIdentifier {
        let id = self.world_id_generator.get_new_id();
        let world = World::new(id, name, Rc::downgrade(&self.entity_id_generator));
        self.worlds.push(world);
        id
    }

    pub fn get_world(&self, id: WorldIdentifier) -> Option<&World> {
        if let Some(world) = self.worlds.iter().find(|world| world.id == id) {
            Some(world)
        } else {
            None
        }
    }

    pub fn get_world_mut(&mut self, id: WorldIdentifier) -> Option<&mut World> {
        if let Some(world) = self.worlds.iter_mut().find(|world| world.id == id) {
            Some(world)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_entity() {
        let mut project = Project::new("test project".to_string());

        let world_a = project.create_world("a".to_string());
        if let Some(world) = project.get_world_mut(world_a) {
            let entity = world.create_entity();

            println!("entity created {:?}", entity);

            let entity = world.create_entity();

            println!("entity created {:?}", entity);
        }

        let world_b = project.create_world("b".to_string());
        if let Some(world) = project.get_world_mut(world_b) {
            let entity = world.create_entity();

            println!("entity created {:?}", entity);
        }
    }
}
