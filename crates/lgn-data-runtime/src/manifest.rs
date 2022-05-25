//! Module containing information about compiled assets.

use lgn_content_store::indexing::TreeIdentifier;

/// Identifier for manifest stored in content-store (as a static-index)
pub struct ManifestId(pub TreeIdentifier);
