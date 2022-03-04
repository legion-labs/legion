use lgn_content_store2::{
    ChunkIdentifier, Chunker, ContentAsyncRead, ContentReader, ContentReaderExt, ContentWriter,
    ContentWriterExt, Identifier,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Asset<Metadata> {
    metadata: Metadata,
    data_id: DataIdentifier,
}

impl<Metadata: Clone> Clone for Asset<Metadata> {
    fn clone(&self) -> Self {
        Self {
            metadata: self.metadata.clone(),
            data_id: self.data_id.clone(),
        }
    }
}

/// An identifier for an asset.
///
/// Note: this is not the same as an object identifier. This is a unique
/// identifier for an asset with a given content and metadata.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AssetIdentifier(pub Identifier);

/// An identifier for an asset data.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DataIdentifier(pub ChunkIdentifier);

impl<Metadata> Asset<Metadata>
where
    Metadata: DeserializeOwned,
{
    /// Create a new asset from its identifiers.
    pub fn new(metadata: Metadata, data_id: DataIdentifier) -> Self {
        Self { metadata, data_id }
    }

    /// Create a new asset by first uploading its data.
    ///
    /// # Errors
    ///
    /// Returns an error if the asset cannot be created.
    pub async fn new_from_data(
        provider: impl ContentWriter + Send + Sync,
        metadata: Metadata,
        data: &[u8],
    ) -> Result<Self> {
        let data_id = Chunker::new(provider).write_chunk(data).await?;

        Ok(Self::new(metadata, DataIdentifier(data_id)))
    }

    /// Load an asset from the content-store.
    ///
    /// # Errors
    ///
    /// Returns an error if the asset could not be loaded.
    pub async fn load(
        provider: impl ContentReader + Send + Sync,
        id: &AssetIdentifier,
    ) -> Result<Self> {
        let data = provider.read_content(&id.0).await?;

        Ok(rmp_serde::from_slice(&data)
            .map_err(|err| anyhow::anyhow!("failed to parse asset: {}", err))?)
    }

    /// Get the asset's metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Get a reader to the asset's data.
    ///
    /// # Errors
    ///
    /// Returns an error if the asset's data could not be read.
    pub async fn get_data_reader(
        &self,
        provider: impl ContentReader + Send + Sync,
    ) -> Result<ContentAsyncRead> {
        Chunker::new(provider)
            .get_chunk_reader(&self.data_id.0)
            .await
            .map_err(Error::ContentStore)
    }

    /// Get the asset's data.
    ///
    /// # Errors
    ///
    /// Returns an error if the asset's data could not be read.
    pub async fn get_data(&self, provider: impl ContentReader + Send + Sync) -> Result<Vec<u8>> {
        Chunker::new(provider)
            .read_chunk(&self.data_id.0)
            .await
            .map_err(Error::ContentStore)
    }
}

impl<Metadata> Asset<Metadata>
where
    Metadata: Serialize,
{
    pub fn as_identifier(&self) -> AssetIdentifier {
        AssetIdentifier(Identifier::new(&self.as_vec()))
    }

    fn as_vec(&self) -> Vec<u8> {
        rmp_serde::to_vec(&self).unwrap()
    }

    /// Save the asset to the content-store.
    ///
    /// Note: the data is supposed to be already stored in the content-store.
    ///
    /// # Errors
    ///
    /// Returns an error if the asset could not be saved.
    pub async fn save(
        &self,
        provider: impl ContentWriter + Send + Sync,
    ) -> Result<AssetIdentifier> {
        let data = self.as_vec();
        let id = provider.write_content(&data).await?;

        Ok(AssetIdentifier(id))
    }
}
