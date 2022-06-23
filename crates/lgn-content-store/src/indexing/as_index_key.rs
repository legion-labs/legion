use super::{BasicIndexer, IndexKey};

pub trait AsIndexKey: Into<IndexKey> + From<IndexKey> {
    type Indexer: BasicIndexer + Sync;

    fn new_indexer() -> Self::Indexer;
}
