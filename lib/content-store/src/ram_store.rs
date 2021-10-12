use std::collections::HashMap;

use crate::ContentStore;

/// In-memory [`ContentStore`] implementation.
///
/// Handy for testing purposes.
pub struct RamContentStore {
    storage: HashMap<u128, Vec<u8>>,
}

impl RamContentStore {
    /// Create empty in-memory `ContentStore`.
    pub fn default() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
}

impl ContentStore for RamContentStore {
    fn write(&mut self, id: u128, data: &[u8]) -> Option<()> {
        if self.storage.contains_key(&id) {
            return None;
        }

        self.storage.insert(id, data.to_owned());
        Some(())
    }

    fn read(&self, id: u128) -> Option<Vec<u8>> {
        self.storage.get(&id).cloned()
    }

    fn remove(&mut self, id: u128) {
        self.storage.remove(&id);
    }
}
