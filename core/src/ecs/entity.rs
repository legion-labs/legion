use std::any::TypeId;

use super::ids::IdentifierGenerator;

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
pub type ComponentType = TypeId;
