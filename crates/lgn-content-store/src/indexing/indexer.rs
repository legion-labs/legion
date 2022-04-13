use std::{fmt::Display, str::FromStr};

use async_recursion::async_recursion;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    ContentProvider, ContentReader, ContentReaderExt, ContentWriter, ContentWriterExt, Error,
    Identifier, Result,
};

use super::{tree::TreeIdentifier, IndexKey, StaticIndexer, TreeLeafNode};

/// Represents an index identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IndexerIdentifier(Identifier);

impl Display for IndexerIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for IndexerIdentifier {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self(s.parse()?))
    }
}

/// An indexer of resources.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Indexer {
    // A single-level index of resources that uses index keys with a static
    // size that can never change.
    //
    // Typically used for resources that can be indexed by a single key, like an
    // `UUID-based index` or a single field search.
    Static(StaticIndexer),

    // A multi-level index of resources.
    //
    // Typically used for multi-level indexing, like dependency trees or
    // multi-coordinate indexes.
    Multi { first: Box<Self>, second: Box<Self> },
}

impl Indexer {
    #[async_recursion]
    pub async fn get_leaf(
        &self,
        content_provider: impl ContentProvider + Send + Sync + 'async_recursion,
        tree_root: &TreeIdentifier,
        index_key: &[IndexKey],
    ) -> Result<Option<TreeLeafNode>> {
        match self {
            Self::Static(indexer) => {
                let index_key = match index_key.len() {
                    1 => &index_key[0],
                    _ => {
                        return Err(crate::Error::InvalidIndexKey(format!(
                            "expected a single key, got {}",
                            index_key.len()
                        )))
                    }
                };

                indexer
                    .get_leaf(content_provider, tree_root, index_key)
                    .await
            }
            Self::Multi { first, second } => {
                if index_key.len() < 2 {
                    return Err(crate::Error::InvalidIndexKey(format!(
                        "expected at least a 2-parts key, got {}",
                        index_key.len()
                    )));
                };

                let tree_id = match first
                    .get_leaf(&content_provider, tree_root, &index_key[0..=0])
                    .await?
                {
                    Some(TreeLeafNode::TreeRoot(tree_id)) => tree_id,
                    Some(TreeLeafNode::Resource(_)) => {
                        return Err(Error::CorruptedTree(format!(
                            "expected a tree, got a resource: {}",
                            index_key[0]
                        )))
                    }
                    None => return Ok(None),
                };

                second
                    .get_leaf(content_provider, &tree_id, &index_key[1..])
                    .await
            }
        }
    }

    fn as_vec(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).unwrap()
    }

    fn from_slice(buf: &[u8]) -> Result<Self> {
        Ok(rmp_serde::from_slice(buf)?)
    }
}

#[async_trait]
pub trait IndexerReader: ContentReader + Send + Sync {
    async fn read_indexer(&self, id: &IndexerIdentifier) -> Result<Indexer> {
        let buf = self.read_content(&id.0).await?;

        Indexer::from_slice(&buf)
    }
}

#[async_trait]
impl<T: ContentReader + Send + Sync> IndexerReader for T {}

#[async_trait]
pub trait IndexerWriter: ContentWriter + Send + Sync {
    async fn write_indexer(&self, indexer: &Indexer) -> Result<IndexerIdentifier> {
        let buf = indexer.as_vec();

        self.write_content(&buf).await.map(IndexerIdentifier)
    }
}

#[async_trait]
impl<T: ContentWriter + Send + Sync> IndexerWriter for T {}
