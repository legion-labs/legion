use std::{
    collections::{BTreeMap, BTreeSet},
    pin::Pin,
    sync::Arc,
};

use async_trait::async_trait;
use futures::future::join_all;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{Error, Identifier, Result};

/// A reader as returned by the `ContentReader` trait.
pub type ContentAsyncRead = Pin<Box<dyn AsyncRead + Send>>;

/// A writer as returned by the `ContentWriter` trait.
pub type ContentAsyncWrite = Pin<Box<dyn AsyncWrite + Send>>;

/// AliasResolver is a trait for resolving aliases from a content-store alias database.
#[async_trait]
pub trait AliasResolver {
    /// Returns the content-store identifier for a given alias.
    ///
    /// If no identifier is found for the specified `key` in the specified
    /// `key_space`, `Error::NotFound` is returned.
    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier>;
}

/// AliasRegisterer is a trait for recording aliases to a content-store alias database.
#[async_trait]
pub trait AliasRegisterer {
    /// Registers a given alias to a content-store identifier.
    ///
    /// The caller must guarantee that the `key` is unique within the specified
    /// `key_space`.
    ///
    /// If an alias already exists with that `key` and `key_space`, `Error::AlreadyExists`
    /// is returned.
    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()>;
}

/// `AliasProvider` is trait for all types that are both alias registerer and resolvers.
pub trait AliasProvider: AliasRegisterer + AliasResolver {}

/// Blanket implementation of `AliasProvider`.
impl<T> AliasProvider for T where T: AliasRegisterer + AliasResolver {}

/// ContentReader is a trait for reading content from a content-store.
#[async_trait]
pub trait ContentReader {
    /// Returns an async reader that reads the content referenced by the
    /// specified identifier.
    ///
    /// If the identifier does not match any content, `Error::NotFound` is
    /// returned.
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead>;

    /// Returns an async reader for each of the specified identifiers.
    ///
    /// If the content for a given identifier does not exist, `Error::NotFound`
    /// is returned instead.
    ///
    /// If the high-level request fails, an error is returned.
    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncRead>>>;
}

/// A default implementation for the `get_content_readers` method that just
/// calls in parallel `get_content_reader` for each identifier.
pub(crate) async fn get_content_readers_impl<'ids>(
    reader: &(dyn ContentReader + Send + Sync),
    ids: &'ids BTreeSet<Identifier>,
) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncRead>>> {
    let futures = ids
        .iter()
        .map(|id| async move { (id, reader.get_content_reader(id).await) })
        .collect::<Vec<_>>();

    Ok(join_all(futures).await.into_iter().collect())
}

#[async_trait]
pub trait AliasContentReaderExt: ContentReaderExt + AliasResolver {
    /// Read the content referenced by the specified identifier.
    async fn get_alias_reader(&self, key_space: &str, key: &str) -> Result<ContentAsyncRead> {
        let id = self.resolve_alias(key_space, key).await?;

        self.get_content_reader(&id).await
    }

    /// Read the content referenced by the specified identifier.
    async fn read_alias(&self, key_space: &str, key: &str) -> Result<Vec<u8>> {
        let id = self.resolve_alias(key_space, key).await?;

        self.read_content(&id).await
    }
}

impl<T: ContentReaderExt + AliasResolver> AliasContentReaderExt for T {}

#[async_trait]
pub trait ContentReaderExt: ContentReader {
    /// Read the content referenced by the specified identifier.
    async fn read_content(&self, id: &Identifier) -> Result<Vec<u8>> {
        if let Identifier::Data(data) = id {
            return Ok(data.to_vec());
        }

        let mut reader = self.get_content_reader(id).await?;

        let mut result = Vec::with_capacity(id.data_size());

        reader
            .read_to_end(&mut result)
            .await
            .map_err(|err| anyhow::anyhow!("failed to read content: {}", err).into())
            .map(|_| result)
    }

    /// Read the contents referenced by the specified identifiers.
    async fn read_contents<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<Vec<u8>>>> {
        let readers = self.get_content_readers(ids).await?;
        let futures = readers
            .into_iter()
            .map(|(id, r)| async move {
                (
                    id,
                    match r {
                        Ok(mut reader) => {
                            let mut result = Vec::with_capacity(id.data_size());
                            reader
                                .read_to_end(&mut result)
                                .await
                                .map_err(|err| {
                                    anyhow::anyhow!("failed to read content for `{}`: {}", id, err)
                                        .into()
                                })
                                .map(|_| result)
                        }
                        Err(err) => Err(err),
                    },
                )
            })
            .collect::<Vec<_>>();

        Ok(join_all(futures).await.into_iter().collect())
    }
}

impl<T: ContentReader> ContentReaderExt for T {}

/// ContentWriter is a trait for writing content to a content-store.
#[async_trait]
pub trait ContentWriter {
    /// Returns an async write to which the content referenced by the specified
    /// specified identifier can be written.
    ///
    /// # Note
    ///
    /// The caller is responsible for writing exactly the data that matches with
    /// the identifier.
    ///
    /// This call in only intended for low-level efficient operations and very
    /// error prone. If possible, use the `write_content` method instead.
    ///
    /// # Errors
    ///
    /// If the data already exists, `Error::AlreadyExists` is returned and the
    /// caller should consider that the write operation is not necessary.
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite>;
}

#[async_trait]
pub trait AliasContentWriterExt: ContentWriterExt + AliasRegisterer {
    /// Get a write for the content referenced by the specified identifier.
    async fn write_alias(&self, key_space: &str, key: &str, data: &[u8]) -> Result<Identifier> {
        let id = self.write_content(data).await?;

        match self.register_alias(key_space, key, &id).await {
            Ok(_) | Err(Error::AlreadyExists) => Ok(id),
            Err(err) => Err(err),
        }
    }
}

impl<T: ContentWriterExt + AliasRegisterer> AliasContentWriterExt for T {}

#[async_trait]
pub trait ContentWriterExt: ContentWriter {
    /// Write the specified content and returns the newly associated identifier.
    ///
    /// If the content already exists, the write is a no-op and no error is
    /// returned.
    async fn write_content(&self, data: &[u8]) -> Result<Identifier> {
        let id = Identifier::new(data);

        if id.is_data() {
            return Ok(id);
        }

        let mut writer = match self.get_content_writer(&id).await {
            Ok(writer) => writer,
            Err(Error::AlreadyExists) => return Ok(id),
            Err(err) => return Err(err),
        };

        writer
            .write_all(data)
            .await
            .map_err(|err| anyhow::anyhow!("failed to write content: {}", err))?;

        writer
            .shutdown()
            .await
            .map_err(|err| anyhow::anyhow!("failed to flush content: {}", err).into())
            .map(|_| id)
    }
}

impl<T: ContentWriter> ContentWriterExt for T {}

/// `ContentProvider` is trait for all types that are both readers and writers.
pub trait ContentProvider: ContentReader + ContentWriter {}

/// Blanket implementation of `ContentProvider`.
impl<T> ContentProvider for T where T: ContentReader + ContentWriter {}

/// Provides addresses for content.
#[async_trait]
pub trait ContentAddressReader {
    /// Returns the address of the content referenced by the specified identifier.
    ///
    /// # Errors
    ///
    /// If the identifier does not match any content, `Error::NotFound` is
    /// returned.
    async fn get_content_read_address(&self, id: &Identifier) -> Result<String>;
}

/// Provides addresses for content.
#[async_trait]
pub trait ContentAddressWriter {
    /// Returns the address of the content referenced by the specified identifier.
    ///
    /// # Errors
    ///
    /// If the identifier already exists, `Error::AlreadyExists` is returned.
    async fn get_content_write_address(&self, id: &Identifier) -> Result<String>;
}

/// `ContentAddressProvider` is trait for all types that are both address readers and writers.
pub trait ContentAddressProvider: ContentAddressReader + ContentAddressWriter {}

/// Blanket implementation of `ContentAddressProvider`.
impl<T> ContentAddressProvider for T where T: ContentAddressReader + ContentAddressWriter {}

/// Blanket implementations for Arc<T> variants.

#[async_trait]
impl<T: AliasResolver + Send + Sync> AliasResolver for Arc<T> {
    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        self.as_ref().resolve_alias(key_space, key).await
    }
}

#[async_trait]
impl<T: AliasRegisterer + Send + Sync> AliasRegisterer for Arc<T> {
    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        self.as_ref().register_alias(key_space, key, id).await
    }
}

#[async_trait]
impl<T: ContentReader + Send + Sync> ContentReader for Arc<T> {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        self.as_ref().get_content_reader(id).await
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncRead>>> {
        self.as_ref().get_content_readers(ids).await
    }
}

#[async_trait]
impl<T: ContentWriter + Send + Sync> ContentWriter for Arc<T> {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        self.as_ref().get_content_writer(id).await
    }
}

#[async_trait]
impl<T: ContentAddressReader + Send + Sync> ContentAddressReader for Arc<T> {
    async fn get_content_read_address(&self, id: &Identifier) -> Result<String> {
        self.as_ref().get_content_read_address(id).await
    }
}

#[async_trait]
impl<T: ContentAddressWriter + Send + Sync> ContentAddressWriter for Arc<T> {
    async fn get_content_write_address(&self, id: &Identifier) -> Result<String> {
        self.as_ref().get_content_write_address(id).await
    }
}

/// Blanket implementations for Box<T> variants.

#[async_trait]
impl<T: AliasResolver + Send + Sync> AliasResolver for Box<T> {
    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        self.as_ref().resolve_alias(key_space, key).await
    }
}

#[async_trait]
impl<T: AliasRegisterer + Send + Sync> AliasRegisterer for Box<T> {
    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        self.as_ref().register_alias(key_space, key, id).await
    }
}

#[async_trait]
impl<T: ContentReader + Send + Sync + ?Sized> ContentReader for Box<T> {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        self.as_ref().get_content_reader(id).await
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncRead>>> {
        self.as_ref().get_content_readers(ids).await
    }
}

#[async_trait]
impl<T: ContentWriter + Send + Sync + ?Sized> ContentWriter for Box<T> {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        self.as_ref().get_content_writer(id).await
    }
}

#[async_trait]
impl<T: ContentAddressReader + Send + Sync + ?Sized> ContentAddressReader for Box<T> {
    async fn get_content_read_address(&self, id: &Identifier) -> Result<String> {
        self.as_ref().get_content_read_address(id).await
    }
}

#[async_trait]
impl<T: ContentAddressWriter + Send + Sync + ?Sized> ContentAddressWriter for Box<T> {
    async fn get_content_write_address(&self, id: &Identifier) -> Result<String> {
        self.as_ref().get_content_write_address(id).await
    }
}

/// Blanket implementations for &T variants.

#[async_trait]
impl<T: AliasResolver + Send + Sync> AliasResolver for &T {
    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        (**self).resolve_alias(key_space, key).await
    }
}

#[async_trait]
impl<T: AliasRegisterer + Send + Sync> AliasRegisterer for &T {
    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        (**self).register_alias(key_space, key, id).await
    }
}

#[async_trait]
impl<T: ContentReader + Send + Sync + ?Sized> ContentReader for &T {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        (**self).get_content_reader(id).await
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncRead>>> {
        (**self).get_content_readers(ids).await
    }
}

#[async_trait]
impl<T: ContentWriter + Send + Sync + ?Sized> ContentWriter for &T {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        (**self).get_content_writer(id).await
    }
}

#[async_trait]
impl<T: ContentAddressReader + Send + Sync + ?Sized> ContentAddressReader for &T {
    async fn get_content_read_address(&self, id: &Identifier) -> Result<String> {
        (**self).get_content_read_address(id).await
    }
}

#[async_trait]
impl<T: ContentAddressWriter + Send + Sync + ?Sized> ContentAddressWriter for &T {
    async fn get_content_write_address(&self, id: &Identifier) -> Result<String> {
        (**self).get_content_write_address(id).await
    }
}
