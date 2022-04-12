//! A set of structures and functions to store and index assets.

mod index;
mod resource;
mod tree;

pub use index::{Index, KeyGetter, KeyPathSplitter};
pub use resource::{Resource, ResourceIdentifier};
pub use tree::{MultiResourcesTree, Tree, TreeNode, UniqueResourceTree};
