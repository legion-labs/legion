use lgn_core::prelude::*;
use lgn_ecs::{prelude::*, world::EntityMut};
use rune::{runtime::Protocol, Any, ContextError, Module};

use super::transform::Transform;

#[derive(Any)]
pub(crate) struct Entity {
    world: *mut World,
    entity: lgn_ecs::prelude::Entity,
}

impl Entity {
    pub(crate) fn new(world: *mut World, entity: lgn_ecs::prelude::Entity) -> Self {
        Self { world, entity }
    }

    pub(crate) fn get_mut(&self) -> EntityMut<'_> {
        #![allow(unsafe_code)]
        let world = unsafe { &mut *self.world };
        world.entity_mut(self.entity)
    }
}

#[derive(Any)]
pub(crate) struct EntityLookupByName {
    world: *mut World,
}

impl EntityLookupByName {
    pub(crate) fn new(world: *mut World) -> Self {
        Self { world }
    }

    fn lookup(&self, entity_name: &str) -> Option<Entity> {
        #![allow(unsafe_code)]
        let world = unsafe { &mut *self.world };

        let mut query = world.query::<(lgn_ecs::prelude::Entity, &Name)>();
        let entity_name: Name = entity_name.into();

        for (entity, name) in query.iter(world) {
            if entity_name == *name {
                return Some(Entity::new(self.world, entity));
            }
        }

        None
    }
}

pub(crate) fn make_ecs_module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("lgn_ecs");

    module.ty::<Entity>()?;
    module.field_fn(Protocol::GET, "transform", |entity: &Entity| {
        Transform::new(entity)
    })?;

    module.ty::<EntityLookupByName>()?;
    module.inst_fn(Protocol::INDEX_GET, EntityLookupByName::lookup)?;

    Ok(module)
}
