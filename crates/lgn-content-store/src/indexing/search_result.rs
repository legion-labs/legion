use super::{tree::TreeLeafNode, IndexPath, TreeIdentifier, TreeNode};

/// An index path search result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SearchResult<'k> {
    /// The search was unsuccessful.
    ///
    /// The last item in the path represents the missing key in its associated
    /// tree. Any previous items in the path are guaranteed to exist.
    ///
    /// The index path is guaranteed to be non-empty when this variant is
    /// returned.
    NotFound(IndexPath<'k>),

    /// The search was successful and a leaf node was found.
    ///
    /// The last item in the path represents the found key in its associated
    /// tree.
    Leaf(IndexPath<'k>, TreeLeafNode),

    /// The search was successful and a branch was found.
    ///
    /// The last item in the path represents the found key in its associated
    /// tree.
    Branch(IndexPath<'k>, TreeIdentifier),
}

impl From<SearchResult<'_>> for Option<TreeNode> {
    fn from(v: SearchResult<'_>) -> Self {
        match v {
            SearchResult::NotFound(_) => None,
            SearchResult::Leaf(_, leaf) => Some(TreeNode::Leaf(leaf)),
            SearchResult::Branch(_, id) => Some(TreeNode::Branch(id)),
        }
    }
}
