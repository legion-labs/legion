use std::any::TypeId;

use super::ids::IdentifierGenerator;
use super::reflection;

pub struct Entity(EntityIdentifier);

impl From<EntityIdentifier> for Entity {
    fn from(id: EntityIdentifier) -> Self {
        Self(id)
    }
}

impl From<Entity> for EntityIdentifier {
    fn from(entity: Entity) -> Self {
        entity.0
    }
}

pub type EntityIdentifier = u64;
pub const INVALID_ENTITY_ID: EntityIdentifier = EntityIdentifier::MAX;

pub type EntityIdentifierGenerator = IdentifierGenerator<EntityIdentifier>;

pub trait Component {}

#[derive(Debug)]
pub struct ComponentType {
    name: &'static str,
    id: TypeId,
}

impl ComponentType {
    pub fn new<T: 'static>() -> Self
    where
        T: 'static + reflection::Named,
    {
        Self {
            name: reflection::get_short_type_name(T::get_name()),
            id: TypeId::of::<T>(),
        }
    }
}
