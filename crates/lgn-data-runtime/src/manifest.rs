//! Module containing information about compiled assets.

use std::ops::Deref;

use lgn_content_store::indexing::TreeIdentifier;

/// Identifier for manifest stored in content-store (as a static-index)
pub struct ManifestId(pub TreeIdentifier);

impl Deref for ManifestId {
    type Target = TreeIdentifier;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
