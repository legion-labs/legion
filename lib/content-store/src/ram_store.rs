use std::collections::HashMap;

use crate::{Checksum, ContentStore};

/// In-memory [`ContentStore`] implementation.
///
/// Handy for testing purposes.
pub struct RamContentStore {
    storage: HashMap<Checksum, Vec<u8>>,
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
    fn write(&mut self, id: Checksum, data: &[u8]) -> Option<()> {
        if self.storage.contains_key(&id) {
            return None;
        }

        self.storage.insert(id, data.to_owned());
        Some(())
    }

    fn read(&self, id: Checksum) -> Option<Vec<u8>> {
        self.storage.get(&id).cloned()
    }

    fn remove(&mut self, id: Checksum) {
        self.storage.remove(&id);
    }
}
