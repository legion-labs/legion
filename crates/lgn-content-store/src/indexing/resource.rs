use crate::{
    ChunkIdentifier, Chunker, ContentAsyncRead, ContentReader, ContentReaderExt, ContentWriter,
    Identifier,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::Result;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Resource<Metadata> {
    metadata: Metadata,
    chunk_id: ChunkIdentifier,
}

impl<Metadata: Clone> Clone for Resource<Metadata> {
    fn clone(&self) -> Self {
        Self {
            metadata: self.metadata.clone(),
            chunk_id: self.chunk_id.clone(),
        }
    }
}

/// An identifier for an resource.
///
/// Note: this is not the same as an object identifier. This is a unique
/// identifier for an resource with a given content and metadata.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ResourceIdentifier(pub Identifier);

impl<Metadata> Resource<Metadata>
where
    Metadata: DeserializeOwned,
{
    /// Create a new resource from its identifiers.
    pub fn new(metadata: Metadata, chunk_id: ChunkIdentifier) -> Self {
        Self { metadata, chunk_id }
    }

    /// Create a new resource by first uploading its data.
    ///
    /// # Errors
    ///
    /// Returns an error if the resource cannot be created.
    pub async fn new_from_data(
        provider: impl ContentWriter + Send + Sync,
        metadata: Metadata,
        data: &[u8],
    ) -> Result<Self> {
        let chunk_id = Chunker::default().write_chunk(provider, data).await?;

        Ok(Self::new(metadata, chunk_id))
    }

    /// Load an resource from the content-store.
    ///
    /// # Errors
    ///
    /// Returns an error if the resource could not be loaded.
    pub async fn load(
        provider: impl ContentReader + Send + Sync,
        id: &ResourceIdentifier,
    ) -> Result<Self> {
        let data = provider.read_content(&id.0).await?;

        Ok(rmp_serde::from_slice(&data)
            .map_err(|err| anyhow::anyhow!("failed to parse resource: {}", err))?)
    }

    /// Get the resource's metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Get a reader to the resource's data.
    ///
    /// # Errors
    ///
    /// Returns an error if the resource's data could not be read.
    pub async fn get_data_reader(
        &self,
        provider: impl ContentReader + Send + Sync,
    ) -> Result<ContentAsyncRead> {
        Chunker::default()
            .get_chunk_reader(provider, &self.chunk_id)
            .await
    }

    /// Get the resource's data.
    ///
    /// # Errors
    ///
    /// Returns an error if the resource's data could not be read.
    pub async fn get_data(&self, provider: impl ContentReader + Send + Sync) -> Result<Vec<u8>> {
        Chunker::default()
            .read_chunk(provider, &self.chunk_id)
            .await
    }
}

impl<Metadata> Resource<Metadata>
where
    Metadata: Serialize,
{
    pub fn as_identifier(&self) -> ResourceIdentifier {
        ResourceIdentifier(Identifier::new(&self.as_vec()))
    }

    pub(crate) fn as_vec(&self) -> Vec<u8> {
        rmp_serde::to_vec(&self).unwrap()
    }
}
