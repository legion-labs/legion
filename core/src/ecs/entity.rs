use super::ids::IdentifierGenerator;

pub struct Entity(EntityIdentifier);

impl Entity {
    pub fn new(id: EntityIdentifier) -> Self {
        Self(id)
    }
}

pub type EntityIdentifier = u64;
pub const INVALID_ENTITY_ID: EntityIdentifier = EntityIdentifier::MAX;

pub type EntityIdentifierGenerator = IdentifierGenerator<EntityIdentifier>;

pub trait Component {}
