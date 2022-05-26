//! Module containing information about compiled assets.

use std::sync::RwLock;

use lgn_content_store::indexing::TreeIdentifier;

/// Identifier for manifest stored in content-store (as a static-index)
pub struct ManifestId(RwLock<TreeIdentifier>);

impl ManifestId {
    /// Create a new `ManifestId` that can be used to share its contents
    pub fn new(tree_id: TreeIdentifier) -> Self {
        Self(RwLock::new(tree_id))
    }

    /// Retrieve manifest id
    pub fn read(&self) -> TreeIdentifier {
        self.0.read().expect("lock is poisoned").clone()
    }

    /// Update manifest id
    pub fn write(&self, tree_id: &TreeIdentifier) {
        *self.0.write().expect("lock is poisoned") = tree_id.clone();
    }
}
