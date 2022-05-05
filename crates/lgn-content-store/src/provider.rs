use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{Debug, Display},
};

use async_recursion::async_recursion;
use futures::future::join_all;
use lgn_tracing::{async_span_scope, debug, span_fn};
use tokio::io::AsyncReadExt;

use crate::{
    Alias, AliasProvider, ContentAsyncRead, ContentAsyncReadWithOriginAndSize, ContentProvider,
    ContentProviderMonitor, ContentReader, ContentReaderExt, ContentWriterExt, Error, HashRef,
    Identifier, Manifest, MemoryAliasProvider, MemoryContentProvider, Origin, Result,
    TransferCallbacks, WithOriginAndSize,
};

/// A `Provider` is a provider that implements the small-content optimization or delegates to a specified provider.
#[derive(Debug)]
pub struct Provider {
    content_provider: ContentProviderMonitor<Box<dyn ContentProvider>>,
    alias_provider: Box<dyn AliasProvider>,
    chunk_size: usize,
}

pub const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024 * 8; // 8MB

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "content-provider: {}, alias-provider: {}",
            self.content_provider, self.alias_provider
        )
    }
}

impl Provider {
    /// Instantiate a new provider that uses the in-memory content provider and
    /// the in-memory alias provider.
    ///
    /// This is mostly useful for tests.
    pub fn new_in_memory() -> Self {
        Self::new(MemoryContentProvider::new(), MemoryAliasProvider::new())
    }

    /// Instantiate a new small content provider that wraps the specified
    /// provider using the default identifier size threshold.
    pub fn new(
        content_provider: impl ContentProvider + 'static,
        alias_provider: impl AliasProvider + 'static,
    ) -> Self {
        Self {
            content_provider: ContentProviderMonitor::new(Box::new(content_provider)),
            alias_provider: Box::new(alias_provider),
            chunk_size: DEFAULT_CHUNK_SIZE, // 8MB
        }
    }

    /// Set the chunk size of this provider.
    pub fn set_chunk_size(&mut self, chunk_size: usize) -> &mut Self {
        self.chunk_size = chunk_size;
        self
    }

    /// Clear the download callbacks.
    pub fn clear_download_callbacks(&mut self) -> &mut Self {
        self.content_provider.clear_download_callbacks();
        self
    }

    /// Set the download callbacks.
    pub fn set_download_callbacks(
        &mut self,
        callbacks: impl TransferCallbacks<HashRef> + 'static,
    ) -> &mut Self {
        self.content_provider.set_download_callbacks(callbacks);
        self
    }

    /// Clear the upload callbacks.
    pub fn clear_upload_callbacks(&mut self) -> &mut Self {
        self.content_provider.clear_upload_callbacks();
        self
    }

    /// Set the upload callbacks.
    pub fn set_upload_callbacks(
        &mut self,
        callbacks: impl TransferCallbacks<HashRef> + 'static,
    ) -> &mut Self {
        self.content_provider.set_upload_callbacks(callbacks);
        self
    }

    /// Check whether a content exists.
    ///
    /// # Errors
    ///
    /// If the identifier existence cannot be determined, an error is returned.
    #[async_recursion]
    pub async fn exists(&self, id: &Identifier) -> Result<bool> {
        async_span_scope!("Provider::exists");

        match id {
            Identifier::Data(_) => Ok(true),
            Identifier::HashRef(id) => Ok(self.content_provider.exists(id).await?),
            Identifier::ManifestRef(size, id) => {
                match self
                    .get_manifest_reader(
                        (*size)
                            .try_into()
                            .expect("size must be convertible to usize"),
                        id,
                    )
                    .await
                {
                    Ok(_) => Ok(true),
                    Err(Error::IdentifierNotFound(_)) => Ok(false),
                    Err(err) => Err(err),
                }
            }
            Identifier::Alias(key) => match self.alias_provider.resolve_alias(key).await {
                Ok(id) => self.exists(&id).await,
                Err(crate::alias_providers::Error::AliasNotFound(_)) => Ok(false),
                Err(err) => Err(err.into()),
            },
        }
    }

    /// Get a reader for the specified identifier.
    ///
    /// # Errors
    ///
    /// If the identifier is not found, an error of type
    /// `Error::IdentifierNotFound` is returned.
    #[async_recursion]
    pub async fn get_reader(&self, id: &Identifier) -> Result<ContentAsyncReadWithOriginAndSize> {
        async_span_scope!("Provider::get_reader");

        match id {
            Identifier::Data(data) => {
                debug!(
                    "Provider::get_reader({}) -> returning data contained in the identifier",
                    id
                );

                Ok(std::io::Cursor::new(data.to_vec())
                    .with_origin_and_size(Origin::InIdentifier {}, data.len()))
            }
            Identifier::HashRef(id) => {
                debug!("Provider::get_reader({}) -> calling the inner provider", id);

                Ok(self.content_provider.get_content_reader(id).await?)
            }
            Identifier::ManifestRef(size, id) => {
                self.get_manifest_reader(
                    (*size)
                        .try_into()
                        .expect("size must be convertible to usize"),
                    id,
                )
                .await
            }
            Identifier::Alias(key) => {
                let id = self.alias_provider.resolve_alias(key).await?;

                self.get_reader(&id).await
            }
        }
    }

    /// Read the size of the specified content.
    ///
    /// For most identifiers, this is a very fast operation that does not go to
    /// the network, as the size is contained in the identifier.
    ///
    /// For aliases identifiers though, the alias needs to be resolved, possibly
    /// recursively to be able to determine the size of the referenced content.
    ///
    /// # Errors
    ///
    /// If the identifier is not found, an error of type
    /// `Error::IdentifierNotFound` is returned.
    pub async fn read_size(&self, id: &Identifier) -> Result<usize> {
        async_span_scope!("Provider::read_size");

        Ok(match id {
            Identifier::Data(data) => data.len(),
            Identifier::HashRef(id) => id.data_size(),
            Identifier::ManifestRef(size, _) => (*size)
                .try_into()
                .expect("size must be convertible to usize"),
            Identifier::Alias(_) => self.get_reader(id).await?.size(),
        })
    }

    /// Get readers for all the specified identifiers.
    ///
    /// # Errors
    ///
    /// If the readers could not be created, an error is returned.
    pub async fn get_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncReadWithOriginAndSize>>> {
        async_span_scope!("Provider::get_readers");

        let futures = ids
            .iter()
            .map(|id| async move { (id, self.get_reader(id).await) })
            .collect::<Vec<_>>();

        Ok(join_all(futures).await.into_iter().collect())
    }

    /// Read the content referenced by the specified identifier with its origin.
    ///
    /// # Errors
    ///
    /// If the identifier is not found, an error of type
    /// `Error::IdentifierNotFound` is returned.
    #[async_recursion]
    pub async fn read_with_origin(&self, id: &Identifier) -> Result<(Vec<u8>, Origin)> {
        async_span_scope!("Provider::read_with_origin");

        match id {
            Identifier::Data(data) => Ok((data.clone().into_vec(), Origin::InIdentifier {})),
            Identifier::HashRef(id) => {
                Ok(self.content_provider.read_content_with_origin(id).await?)
            }
            Identifier::ManifestRef(size, id) => {
                let mut reader = self
                    .get_manifest_reader(
                        (*size)
                            .try_into()
                            .expect("size must be convertible to usize"),
                        id,
                    )
                    .await?;

                let mut result = Vec::with_capacity(
                    (*size)
                        .try_into()
                        .expect("size must be convertible to usize"),
                );

                reader
                    .read_to_end(&mut result)
                    .await
                    .map_err(|err| anyhow::anyhow!("failed to read content: {}", err).into())
                    .map(|_| (result, reader.origin().clone()))
            }
            Identifier::Alias(key) => {
                let id = self.alias_provider.resolve_alias(key).await?;

                self.read_with_origin(&id).await
            }
        }
    }

    /// Read the content referenced by the specified identifier.
    ///
    /// # Errors
    ///
    /// If the identifier is not found, an error of type
    /// `Error::IdentifierNotFound` is returned.
    pub async fn read(&self, id: &Identifier) -> Result<Vec<u8>> {
        async_span_scope!("Provider::read");

        self.read_with_origin(id).await.map(|(data, _)| data)
    }

    /// Read the content referenced by the specified alias.
    ///
    /// # Errors
    ///
    /// If the identifier is not found, an error of type
    /// `Error::IdentifierNotFound` is returned.
    pub async fn read_alias(&self, alias: impl Into<Alias>) -> Result<Vec<u8>> {
        async_span_scope!("Provider::read_alias");

        self.read(&Identifier::new_alias(alias.into())).await
    }

    /// Read a manifest stored at the specified identifier.
    ///
    /// # Errors
    ///
    /// If the reader fails to read the manifest, an error is returned.
    #[span_fn]
    async fn read_manifest(&self, id: &Identifier) -> Result<Manifest> {
        debug!("Provider::read_manifest({})", id);

        let mut reader = self.get_reader(id).await?;

        Ok(Manifest::read_from(&mut reader).await?)
    }

    /// Returns an async reader that assembles and reads the manifest referenced
    /// by the specified identifier.
    ///
    /// The final data pointed to by the manifest is expected to be of the exact
    /// specified size.
    ///
    /// # Errors
    ///
    /// If the identifier does not match any content, `Error::NotFound` is
    /// returned.
    ///
    /// If the content referenced by the identifier is not a valid manifest,
    /// `Error::InvalidManifest` is returned.
    #[span_fn]
    async fn get_manifest_reader(
        &self,
        size: usize,
        id: &Identifier,
    ) -> Result<ContentAsyncReadWithOriginAndSize> {
        debug!("Provider::get_chunk_reader({})", id);

        // TODO: This implementation is actually not great:
        //
        // It fetches all the readers in one go but reads them one at a time.
        // This means that the later used readers have all the time in the world
        // to timeout before an actual read is even attempted.
        //
        // It is also not very nice to the backend to spam it with requests all
        // at once.
        //
        // It would be better if we fetched readers as we go along and forgo
        // failing early in favor of more reliable reads.
        //
        // Alternatively, we we could make it so that the HTTP AsyncRead don't
        // actually establish the connection until first polled. That would help
        // too. Not sure if it is possible to do efficiently though.
        //
        // Anthony D.: a task for you? :D

        let manifest = self.read_manifest(id).await?;

        debug!("Provider::get_manifest_reader({}) -> manifest was read", id);

        let ids = manifest.identifiers();
        let mut ids_iter = ids.iter();

        let first_id = match ids_iter.next() {
            Some(id) => id,
            None => {
                return Ok(Box::pin(tokio::io::empty())
                    .with_origin_and_size(Origin::Manifest { id: id.clone() }, size));
            }
        };

        // Get all the necessary readers: if at least one is missing, return the failure.
        let ids_set = &ids.iter().cloned().collect();

        let mut reader_stores = self
            .get_readers(ids_set)
            .await?
            .into_iter()
            .map(|(id, reader)| match reader {
                Ok(reader) => Ok((id, AsyncReadStore::new(reader, size))),
                Err(err) => Err(err),
            })
            .collect::<Result<BTreeMap<_, _>>>()?;

        // Now this is were things get tricky: it's entirely possible that some
        // ids appear in the chunk index more than once.
        //
        // Since readers can only be read once, we need to make sure that the
        // readers for those ids are actually stored in memory the first time
        // they are read, and dropped as soon as they are no longer needed to
        // avoid hogging too much memory.
        //
        // Here we ensure that the `AsyncReadStore` have the appropriate
        // reference counts by doing a first pass over the ids.
        //
        // If an id is to be read several times, it will be read and stored in
        // memory to allow for several reads.

        for id in ids {
            reader_stores
                .get_mut(id)
                .ok_or_else(|| Error::IdentifierNotFound(id.clone()))?
                .inc_ref_count()
                .await?;
        }

        let mut reader = reader_stores.get_mut(first_id).unwrap().get_ref()?;

        for id in ids_iter {
            let next_reader = reader_stores.get_mut(id).unwrap().get_ref()?;
            reader = Box::pin(reader.chain(next_reader));
        }

        debug!(
            "Chunker::get_chunk_reader({}) -> got readers for all chunks",
            id
        );

        Ok(reader.with_origin_and_size(Origin::Manifest { id: id.clone() }, size))
    }

    /// Resolve the specified alias to its identifier.
    ///
    /// # Errors
    ///
    /// If the alias is not found, an error of type `Error::AliasNotFound` is
    /// returned.
    pub async fn resolve_alias(&self, key: &[u8]) -> Result<Identifier> {
        Ok(self.alias_provider.resolve_alias(key).await?)
    }

    /// Compute the identifier for the specified data.
    pub fn compute_id(&self, data: &[u8]) -> Identifier {
        if data.len() > self.chunk_size {
            // The data is bigger than a chunk: let's chunk it.
            self.compute_manifest_id(data)
        } else if data.len() > Identifier::SMALL_IDENTIFIER_SIZE {
            let id = HashRef::new_from_data(data);

            Identifier::new_hash_ref(id)
        } else {
            Identifier::new_data(data)
        }
    }

    /// Write the specified content to the content-store.
    ///
    /// # Errors
    ///
    /// If the content cannot be written, an error is returned.
    pub async fn write(&self, data: &[u8]) -> Result<Identifier> {
        if data.len() > self.chunk_size {
            // The data is bigger than a chunk: let's chunk it.
            self.write_manifest(data).await
        } else if data.len() > Identifier::SMALL_IDENTIFIER_SIZE {
            let id = self.content_provider.write_content(data).await?;

            Ok(Identifier::new_hash_ref(id))
        } else {
            Ok(Identifier::new_data(data))
        }
    }

    /// Unwrite the specified content from the content-store.
    ///
    /// Unwriting doesn't actually delete anything, but in some cases, it
    /// decrements the reference count of the content which can avoid the
    /// content being copied to a more persistent data-store.
    ///
    /// # Errors
    ///
    /// Returns an error if the content cannot be unwritten.
    pub async fn unwrite(&self, _id: &Identifier) -> Result<()> {
        //TODO: Implement.

        Ok(())
    }

    /// Pop the list of identifiers whose reference count is strictly positive.
    ///
    /// After the call, the reference count of the return identifiers will be set to zero.
    #[allow(clippy::unused_self)]
    pub fn pop_referenced_identifiers(&self) -> Vec<Identifier> {
        //TODO: Implement.
        vec![]
    }

    /// Register the specified alias.
    ///
    /// The caller must ensure that the alias is not already registered.
    ///
    /// # Errors
    ///
    /// If the alias is already associated with a content,
    /// `Error::AliasAlreadyExist` is returned.
    pub async fn register_alias(
        &self,
        alias: impl Into<Alias>,
        id: &Identifier,
    ) -> Result<Identifier> {
        Ok(self
            .alias_provider
            .register_alias(&alias.into(), id)
            .await?)
    }

    /// Write the content specified and associate it with the specified alias.
    ///
    /// The caller must ensure that the alias is not already registered.
    ///
    /// # Errors
    ///
    /// If the alias is already associated with a content,
    /// `Error::AliasAlreadyExist` is returned.
    pub async fn write_alias(&self, alias: impl Into<Alias>, data: &[u8]) -> Result<Identifier> {
        let id = self.write(data).await?;

        self.register_alias(alias, &id).await
    }

    /// Get the manifest for the specified data.
    fn compute_manifest_id(&self, data: &[u8]) -> Identifier {
        let mut ids = Vec::with_capacity((data.len() / self.chunk_size) + 1);

        for chunk in data.chunks(self.chunk_size) {
            let id = self.compute_id(chunk);

            ids.push(id);
        }

        // Heuristic to avoid reallocs: probably a bit wasteful but good enough.
        let mut buf = Vec::with_capacity(ids.len() * Identifier::SMALL_IDENTIFIER_SIZE);

        Manifest::Linear(ids).write_all_to(&mut buf).unwrap();

        self.compute_id(&buf)
    }

    /// Writes the specified content to the content store, splitting it into
    /// chunks.
    ///
    /// # Errors
    ///
    /// If the writing fails, an error is returned.
    #[async_recursion]
    async fn write_manifest(&self, data: &[u8]) -> Result<Identifier> {
        let mut ids = Vec::with_capacity((data.len() / self.chunk_size) + 1);

        for chunk in data.chunks(self.chunk_size) {
            let id = self.write(chunk).await?;

            ids.push(id);
        }

        // Heuristic to avoid reallocs: probably a bit wasteful but good enough.
        let mut buf = Vec::with_capacity(ids.len() * Identifier::SMALL_IDENTIFIER_SIZE);

        let manifest = Manifest::Linear(ids);

        match manifest.write_all_to(&mut buf) {
            Ok(()) => self
                .write(&buf)
                .await
                .map(|id| Identifier::new_manifest_ref(data.len(), id)),
            Err(err) => Err(anyhow::anyhow!("failed to write manifest: {}", err).into()),
        }
    }
}

struct AsyncReadStore {
    state: AsyncReadStoreState,
    refs: usize,
    size: usize,
}
enum AsyncReadStoreState {
    Single(Option<ContentAsyncReadWithOriginAndSize>),
    Multi(Option<Vec<u8>>),
}

impl AsyncReadStore {
    pub fn new(reader: ContentAsyncReadWithOriginAndSize, size: usize) -> Self {
        Self {
            state: AsyncReadStoreState::Single(Some(reader)),
            refs: 0,
            size,
        }
    }

    #[allow(clippy::uninit_vec, unsafe_code)]
    pub async fn inc_ref_count(&mut self) -> Result<()> {
        self.refs += 1;

        if self.refs == 2 {
            match &mut self.state {
                AsyncReadStoreState::Single(Some(reader)) => {
                    let mut buf = Vec::with_capacity(self.size);

                    reader
                        .read_to_end(&mut buf)
                        .await
                        .map_err(|err| anyhow::anyhow!("failed to read chunk: {}", err))?;

                    self.state = AsyncReadStoreState::Multi(Some(buf));
                }
                AsyncReadStoreState::Single(None) => {
                    return Err(Error::Unknown(anyhow::anyhow!("reader is None")))
                }
                AsyncReadStoreState::Multi(_) => {}
            };
        }

        Ok(())
    }

    pub fn get_ref(&mut self) -> Result<ContentAsyncRead> {
        if self.refs == 0 {
            return Err(Error::Unknown(anyhow::anyhow!(
                "AsyncReadStore has no references left"
            )));
        }

        self.refs -= 1;

        match &mut self.state {
            AsyncReadStoreState::Single(reader) => {
                if let Some(reader) = reader.take() {
                    // TODO: Remove the Box::pin below once trait upcasting becomes a thing.
                    Ok(Box::pin(reader) as ContentAsyncRead)
                } else {
                    Err(Error::Unknown(anyhow::anyhow!("reader is None")))
                }
            }
            AsyncReadStoreState::Multi(buf) => {
                if self.refs == 0 {
                    if let Some(buf) = buf.take() {
                        Ok(Box::pin(std::io::Cursor::new(buf)) as ContentAsyncRead)
                    } else {
                        Err(Error::Unknown(anyhow::anyhow!("buf is None")))
                    }
                } else if let Some(buf) = buf {
                    Ok(Box::pin(std::io::Cursor::new(buf.clone())) as ContentAsyncRead)
                } else {
                    Err(Error::Unknown(anyhow::anyhow!("buf is None")))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{HashRef, LocalAliasProvider, LocalContentProvider};

    use super::*;

    #[tokio::test]
    async fn test_provider() {
        let root = tempfile::tempdir().expect("failed to create temp directory");
        let content_provider = LocalContentProvider::new(root.path())
            .await
            .expect("failed to create local content-provider");
        let alias_provider = LocalAliasProvider::new(root.path())
            .await
            .expect("failed to create local alias-provider");

        let mut provider = Provider::new(content_provider, alias_provider);
        provider.set_chunk_size(1024);

        const SMALL_DATA_A: &[u8] = &[0x41; 32];
        const BIG_DATA_A: &[u8] = &[0x41; 1024];
        const BIGGER_DATA_A: &[u8] = &[0x41; 1024 * 2 + 16];

        let id = provider.write(SMALL_DATA_A).await.unwrap();
        assert!(id.is_data());

        let (data, origin) = provider.read_with_origin(&id).await.unwrap();
        assert_eq!(&data, SMALL_DATA_A);
        assert_eq!(origin, Origin::InIdentifier {});

        // Another write should yield no error.
        let new_id = provider.write(SMALL_DATA_A).await.unwrap();
        assert_eq!(id, new_id);

        // Now let's try again with a larger file.
        let id = Identifier::new_hash_ref(HashRef::new_from_data(BIG_DATA_A));

        match provider.read(&id).await {
            Ok(_) => panic!("read should have failed"),
            Err(Error::IdentifierNotFound(err_id)) => {
                assert_eq!(err_id, id);
            }
            Err(err) => panic!("unexpected error: {}", err),
        };

        let new_id = provider.write(BIG_DATA_A).await.unwrap();
        assert_eq!(id, new_id);
        assert!(new_id.is_hash_ref());

        let (data, origin) = provider.read_with_origin(&id).await.unwrap();
        assert_eq!(&data, BIG_DATA_A);
        assert_eq!(
            origin,
            Origin::Local {
                path: root.path().join(id.to_string())
            }
        );

        // Now let's try again with an even larger file.
        let id = provider.write(BIGGER_DATA_A).await.unwrap();

        let manifest_id = if let Identifier::ManifestRef(_, manifest_id) = &id {
            *manifest_id.clone()
        } else {
            panic!("expected manifest-ref, got: {:?}", id);
        };

        let (data, origin) = provider.read_with_origin(&id).await.unwrap();
        assert_eq!(&data, BIGGER_DATA_A);
        assert_eq!(origin, Origin::Manifest { id: manifest_id });
    }
}
