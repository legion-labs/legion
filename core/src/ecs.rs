use std::{cell::RefCell, ops::AddAssign};

pub struct Entity {
    id: EntityIdentifier,
}

pub trait Component {}

pub struct World {
    id: WorldIdentifier,
    name: String,
    entities: Vec<Entity>,
    // project: Weak<RefCell<Project>>,
}

impl World {
    pub fn new(id: WorldIdentifier, name: String, project: &Project) -> Self {
        Self {
            id,
            name,
            entities: Vec::new(),
            // project: Rc::downgrade(&project),
        }
    }

    fn create_entity(&mut self, id: EntityIdentifier) -> EntityIdentifier {
        self.entities.push(Entity { id });
        id
    }

    fn create_entity_with_project(&mut self, project: &Project) -> EntityIdentifier {
        self.create_entity(project.get_new_entity_id())
    }
}
trait One {
    fn one() -> Self;
}

pub type EntityIdentifier = u64;
//const INVALID_ENTITY_ID: EntityIdentifier = EntityIdentifier::MAX;

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

struct IdentifierGenerator<T>
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

pub struct Project {
    name: String,
    world_id_generator: IdentifierGenerator<WorldIdentifier>,
    worlds: Vec<World>,
    entity_id_generator: RefCell<IdentifierGenerator<EntityIdentifier>>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            name,
            world_id_generator: IdentifierGenerator::<WorldIdentifier>::new(),
            worlds: Vec::new(),
            entity_id_generator: RefCell::new(IdentifierGenerator::<EntityIdentifier>::new()),
        }
    }

    // World management

    pub fn create_world(&mut self, name: String) -> WorldIdentifier {
        let id = self.world_id_generator.get_new_id();
        let world = World::new(id, name, self);
        self.worlds.push(world);
        id
    }

    pub fn get_world(&self, id: WorldIdentifier) -> Option<&World> {
        if let Some(world) = self.worlds.iter().find(|&world| world.id == id) {
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

    pub fn get_new_entity_id(&self) -> EntityIdentifier {
        self.entity_id_generator.borrow_mut().get_new_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_entity() {
        let mut project = Project::new("test project".to_string());

        {
            let world_id = project.create_world("test world".to_string());

            let entity_id = project.get_new_entity_id();

            if let Some(world) = project.get_world_mut(world_id) {
                //let entity = project.create_entity(world);

                //let entity = world.create_entity_with_project(&project);

                let entity = world.create_entity(entity_id);

                println!("entity created {:?}", entity);
            }
        }

        {
            let _world_id = project.create_world("another world".to_string());
        }
    }
}
