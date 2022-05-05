use std::{fmt::Display, str::FromStr};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{Identifier, Provider};

use super::{
    tree::TreeIdentifier, Error, IndexKey, Indexer, IndexerIdentifier, IndexerReader,
    ResourceIdentifier, Result, TreeLeafNode,
};

/// Represents an index identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IndexIdentifier(pub(crate) Identifier);

impl Display for IndexIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for IndexIdentifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.parse() {
            Ok(id) => Ok(Self(id)),
            Err(err) => Err(Error::InvalidIndexIdentifier(err)),
        }
    }
}

/// An index of resources.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Index {
    #[serde(rename = "i")]
    indexer_id: IndexerIdentifier,
    #[serde(rename = "t")]
    tree_root: TreeIdentifier,
}

impl Index {
    /// Load an index.
    ///
    /// # Errors
    ///
    /// Return an error if the index cannot be loaded.
    pub async fn load(provider: &Provider, id: &IndexIdentifier) -> Result<Self> {
        provider.read_index(id).await
    }

    /// Save an index.
    ///
    /// # Errors
    ///
    /// Return an error if the index cannot be saved.
    pub async fn save(&self, provider: &Provider) -> Result<IndexIdentifier> {
        provider.write_index(self).await
    }

    /// Get a resource from the index by performing an exact search.
    ///
    /// This kind of search is usually pretty efficient and implemented on most
    /// index types.
    ///
    /// This function will return `None` if the index does not contain the
    /// resource with the specified key.
    ///
    /// # Errors
    ///
    /// If the index does not support this operation,
    /// `Error::UnsupportedIndexOperation` is returned.
    ///
    /// If the specified key does not match the indexer type,
    /// `Error::InvalidIndexKey` is returned.
    pub async fn get_resource(
        &self,
        provider: &Provider,
        index_key: &[IndexKey],
    ) -> Result<Option<ResourceIdentifier>> {
        let indexer = self.indexer(provider).await?;

        if let Some(leaf_node) = indexer
            .get_leaf(provider, &self.tree_root, index_key)
            .await?
        {
            match leaf_node {
                TreeLeafNode::Resource(resource_identifier) => Ok(Some(resource_identifier)),
                TreeLeafNode::TreeRoot(tree_identifier) => Err(Error::CorruptedTree(format!(
                    "expected a resource, got a tree: {}",
                    tree_identifier
                ))),
            }
        } else {
            Ok(None)
        }
    }

    async fn indexer(&self, provider: &Provider) -> Result<Indexer> {
        provider.read_indexer(&self.indexer_id).await
    }

    fn as_vec(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).unwrap()
    }

    fn from_slice(buf: &[u8]) -> Result<Self> {
        Ok(rmp_serde::from_slice(buf)?)
    }
}

#[async_trait]
pub trait IndexReader {
    async fn read_index(&self, id: &IndexIdentifier) -> Result<Index>;
}

#[async_trait]
impl IndexReader for Provider {
    async fn read_index(&self, id: &IndexIdentifier) -> Result<Index> {
        let buf = self.read(&id.0).await?;

        Index::from_slice(&buf)
    }
}

#[async_trait]
pub trait IndexWriter {
    async fn write_index(&self, index: &Index) -> Result<IndexIdentifier>;
}

#[async_trait]
impl IndexWriter for Provider {
    async fn write_index(&self, index: &Index) -> Result<IndexIdentifier> {
        let buf = index.as_vec();

        self.write(&buf)
            .await
            .map(IndexIdentifier)
            .map_err(Into::into)
    }
}
