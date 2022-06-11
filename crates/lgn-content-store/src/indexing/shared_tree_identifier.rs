use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};

use super::TreeIdentifier;

/// Tree identifier that can be shared across threads
pub struct SharedTreeIdentifier(Arc<RwLock<TreeIdentifier>>);

impl SharedTreeIdentifier {
    /// Create a new `ManifestId` that can be used to share its contents
    pub fn new(tree_id: TreeIdentifier) -> Self {
        Self(Arc::new(RwLock::new(tree_id)))
    }

    /// Retrieve manifest id
    pub fn read(&self) -> TreeIdentifier {
        self.0.read().expect("lock is poisoned").clone()
    }

    /// Update manifest id
    pub fn write(&self, tree_id: TreeIdentifier) {
        *self.0.write().expect("lock is poisoned") = tree_id;
    }
}

impl Clone for SharedTreeIdentifier {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl Display for SharedTreeIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.read(), f)
    }
}
