use std::iter::Rev;

use smallvec::SmallVec;

use super::Tree;

/// A index path item.
///
/// The `key` always represents a direct children key of the associated `tree`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IndexPathItem<'k> {
    pub tree: Tree,
    pub key: &'k [u8],
}

/// An index path that behaves as a stack.
///
/// Iterating over the path will be done in reverse order, last element first,
/// to ease tree recomputation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct IndexPath<'k>(SmallVec<[IndexPathItem<'k>; 8]>);

impl<'k> IndexPath<'k> {
    pub fn push(&mut self, item: IndexPathItem<'k>) {
        self.0.push(item);
    }

    pub fn pop(&mut self) -> Option<IndexPathItem<'k>> {
        self.0.pop()
    }

    pub fn last_mut(&mut self) -> Option<&mut IndexPathItem<'k>> {
        self.0.last_mut()
    }
}

impl<'k> IntoIterator for IndexPath<'k> {
    type Item = IndexPathItem<'k>;

    type IntoIter = Rev<<SmallVec<[IndexPathItem<'k>; 8]> as IntoIterator>::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().rev()
    }
}
