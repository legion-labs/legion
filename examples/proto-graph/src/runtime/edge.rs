use uuid::Uuid;

pub struct Edge {
    pub from: Uuid,
    pub to: Uuid,
}

impl Edge {
    pub fn new(from: Uuid, to: Uuid) -> Self {
        Self { from, to }
    }
}
