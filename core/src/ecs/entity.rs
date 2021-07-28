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
    pub fn new<T>() -> Self
    where
        T: reflection::Reference,
    {
        Self {
            name: T::get_short_type_name(),
            id: T::get_type_id(),
        }
    }

    pub fn get_name(&self) -> &str {
        self.name
    }

    pub fn get_id(&self) -> &TypeId {
        &self.id
    }
}
