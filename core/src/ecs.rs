use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub type EntityIdentifier = u64;
const INVALID_ID: EntityIdentifier = 0;

pub struct Entity {
    id: EntityIdentifier,
}

pub trait Component {}

pub struct World {
    name: String,
    entities: Vec<Entity>,
    // todo add weak reference to Project
    project_data: Weak<RefCell<ProjectData>>,
}

impl World {
    pub fn new(name: String, project: &Project) -> Self {
        Self {
            name,
            entities: Vec::new(),
            project_data: Rc::downgrade(&project.data),
        }
    }

    fn create_entity(&mut self) -> EntityIdentifier {
        if let Some(project_data) = self.project_data.borrow().upgrade() {
            let id = (*project_data)
                .borrow()
                .id_generator
                .borrow_mut()
                .get_new_id();
            self.entities.push(Entity { id });
            return id;
        }
        INVALID_ID
    }
}

struct EntityIdentifierGenerator {
    next_valid_id: EntityIdentifier,
}

impl EntityIdentifierGenerator {
    fn new() -> Self {
        Self {
            next_valid_id: INVALID_ID,
        }
    }

    fn get_new_id(&mut self) -> EntityIdentifier {
        self.next_valid_id += 1;
        self.next_valid_id
    }
}

struct ProjectData {
    name: String,
    worlds: Vec<World>,
    id_generator: RefCell<EntityIdentifierGenerator>,
}

impl ProjectData {
    pub fn create_world<'w>(&mut self, name: String, project: &'w Project) -> &'w mut World {
        let world = World::new(name, project);
        self.worlds.push(world);
        self.worlds.last_mut().unwrap()
    }
}

struct Project {
    data: Rc<RefCell<ProjectData>>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            data: Rc::new(RefCell::new(ProjectData {
                name,
                worlds: Vec::new(),
                id_generator: RefCell::new(EntityIdentifierGenerator::new()),
            })),
        }
    }

    pub fn create_world(&mut self, name: String) -> &mut World {
        //        let &
        //        let rc_data = (*self.data).borrow_mut();
        //let data = (*self.data).borrow_mut();
        (*self.data).borrow_mut().create_world(name, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_entity() {
        let mut project = Project::new("test project".to_string());

        {
            let world = project.create_world("test world".to_string());

            //let entity = project.create_entity(world);
            let entity = world.create_entity();

            println!("entity created {:?}", entity);
        }

        {
            let _world = project.create_world("another world".to_string());
        }
    }
}
