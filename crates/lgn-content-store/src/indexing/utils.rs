use crate::Provider;

use super::{errors::Result, Tree, TreeIdentifier, TreeWriter};

/// Identifier for an empty tree
///
/// # Errors
///
/// If the tree cannot be written, an error will be returned.
pub async fn empty_tree_id(provider: &Provider) -> Result<TreeIdentifier> {
    provider.write_tree(&Tree::default()).await
}
