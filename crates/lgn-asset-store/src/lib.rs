//! A set of structures and functions to store and index assets.

mod asset;
mod errors;
mod index;
mod tree;

pub use asset::{Asset, AssetIdentifier};
pub use errors::{Error, Result};
pub use index::{Index, KeyGetter, KeyPathSplitter};
pub use tree::{Tree, TreeNode};
