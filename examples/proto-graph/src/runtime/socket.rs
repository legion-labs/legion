use super::types::Type;

use uuid::Uuid;

#[derive(Default)]
pub struct Socket {
    value: Type,
    uuid: Uuid,
}

impl Socket {
    pub fn new(value: Type) -> Self {
        Self {
            value,
            uuid: Uuid::new_v4(),
        }
    }

    pub fn get_value(&self) -> Type {
        self.value.clone()
    }
}
