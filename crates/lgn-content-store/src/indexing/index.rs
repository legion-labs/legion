use std::{fmt::Display, str::FromStr};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    ContentProvider, ContentReader, ContentReaderExt, ContentWriter, ContentWriterExt, Identifier,
    Result,
};

use super::{
    tree::TreeIdentifier, IndexKey, Indexer, IndexerIdentifier, IndexerReader, ResourceIdentifier,
    TreeLeafNode,
};

/// Represents an index identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IndexIdentifier(Identifier);

impl Display for IndexIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for IndexIdentifier {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self(s.parse()?))
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
    pub async fn load(
        content_reader: impl ContentReader + Send + Sync,
        id: &IndexIdentifier,
    ) -> Result<Self> {
        content_reader.read_index(id).await
    }

    /// Save an index.
    ///
    /// # Errors
    ///
    /// Return an error if the index cannot be saved.
    pub async fn save(
        &self,
        content_writer: impl ContentWriter + Send + Sync,
    ) -> Result<IndexIdentifier> {
        content_writer.write_index(self).await
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
        content_provider: impl ContentProvider + Send + Sync,
        index_key: &[IndexKey],
    ) -> Result<Option<ResourceIdentifier>> {
        let indexer = self.indexer(&content_provider).await?;

        if let Some(leaf_node) = indexer
            .get_leaf(content_provider, &self.tree_root, index_key)
            .await?
        {
            match leaf_node {
                TreeLeafNode::Resource(resource_identifier) => Ok(Some(resource_identifier)),
                TreeLeafNode::TreeRoot(tree_identifier) => Err(crate::Error::CorruptedTree(
                    format!("expected a resource, got a tree: {}", tree_identifier),
                )),
            }
        } else {
            Ok(None)
        }
    }

    async fn indexer(&self, content_reader: impl ContentReader + Send + Sync) -> Result<Indexer> {
        content_reader.read_indexer(&self.indexer_id).await
    }

    fn as_vec(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).unwrap()
    }

    fn from_slice(buf: &[u8]) -> Result<Self> {
        Ok(rmp_serde::from_slice(buf)?)
    }
}

#[async_trait]
pub trait IndexReader: ContentReader + Send + Sync {
    async fn read_index(&self, id: &IndexIdentifier) -> Result<Index> {
        let buf = self.read_content(&id.0).await?;

        Index::from_slice(&buf)
    }
}

#[async_trait]
impl<T: ContentReader + Send + Sync> IndexReader for T {}

#[async_trait]
pub trait IndexWriter: ContentWriter + Send + Sync {
    async fn write_index(&self, index: &Index) -> Result<IndexIdentifier> {
        let buf = index.as_vec();

        self.write_content(&buf).await.map(IndexIdentifier)
    }
}

#[async_trait]
impl<T: ContentWriter + Send + Sync> IndexWriter for T {}
