use std::{collections::BTreeMap, io::Write};

use itertools::Itertools;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

use crate::{
    ChunkIdentifier, ContentAsyncRead, ContentReader, ContentWriter, ContentWriterExt, Error,
    Identifier, Result,
};

/// A provider-like type that splits data into chunks and stores them in a
/// content-store.
pub struct Chunker<Provider> {
    provider: Provider,
    chunk_size: usize,
    max_parallel_uploads: usize,
}

impl<Provider> Chunker<Provider> {
    pub const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024 * 32; // 32 MB
    pub const DEFAULT_MAX_PARALLEL_UPLOADS: usize = 8;

    /// Create a new chunker instance that uses the default chunk size.
    pub fn new(provider: Provider) -> Self {
        Self {
            provider,
            chunk_size: Self::DEFAULT_CHUNK_SIZE,
            max_parallel_uploads: Self::DEFAULT_MAX_PARALLEL_UPLOADS,
        }
    }

    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        assert!(chunk_size > 0);
        self.chunk_size = chunk_size;

        self
    }

    pub fn with_max_parallel_uploads(mut self, max_parallel_uploads: usize) -> Self {
        assert!(max_parallel_uploads > 0);
        self.max_parallel_uploads = max_parallel_uploads;

        self
    }

    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    pub fn max_parallel_uploads(&self) -> usize {
        self.max_parallel_uploads
    }
}

impl<Provider: ContentReader> Chunker<Provider> {
    /// Returns an async reader that assembles and reads the chunk content
    /// referenced by the specified identifier.
    ///
    /// # Errors
    ///
    /// If the identifier does not match any content, `Error::NotFound` is
    /// returned.
    ///
    /// If the content referenced by the identifier is not a chunk index,
    /// `Error::InvalidChunkIndex` is returned.
    pub async fn get_chunk_reader(&self, id: &ChunkIdentifier) -> Result<ContentAsyncRead> {
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

        let mut reader = self.provider.get_content_reader(id.content_id()).await?;
        let chunk_index = ChunkIndex::read_from(&mut reader).await?;
        let ids = chunk_index.identifiers();
        let mut ids_iter = ids.iter();

        let first_id = match ids_iter.next() {
            Some(id) => id,
            None => {
                return Ok(Box::pin(tokio::io::empty()));
            }
        };

        // Get all the necessary readers: if at least one is missing, return the failure.
        let ids_set = &ids.iter().cloned().collect();

        let mut reader_stores = self
            .provider
            .get_content_readers(ids_set)
            .await?
            .into_iter()
            .map(|(id, reader)| match reader {
                Ok(reader) => Ok((id, AsyncReadStore::new(reader, id.data_size()))),
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
                .ok_or(Error::NotFound)?
                .inc_ref_count()
                .await?;
        }

        let mut reader = reader_stores.get_mut(first_id).unwrap().get_ref()?;

        for id in ids_iter {
            let next_reader = reader_stores.get_mut(id).unwrap().get_ref()?;
            reader = Box::pin(reader.chain(next_reader));
        }

        Ok(reader)
    }

    /// Read the chunked content referenced by the specified identifier.
    ///
    /// # Errors
    ///
    /// If the identifier does not match any content, `Error::NotFound` is
    /// returned.
    pub async fn read_chunk(&self, id: &ChunkIdentifier) -> Result<Vec<u8>> {
        let mut reader = self.get_chunk_reader(id).await?;

        let mut result = Vec::with_capacity(id.data_size());

        reader
            .read_to_end(&mut result)
            .await
            .map_err(|err| anyhow::anyhow!("failed to read chunk: {}", err).into())
            .map(|_| result)
    }
}

impl<Provider: ContentWriter + Send + Sync> Chunker<Provider> {
    /// Writes the specified content to the content store, splitting it into
    /// chunks.
    ///
    /// # Errors
    ///
    /// If the writing fails, an error is returned.
    pub async fn write_chunk(&self, data: &[u8]) -> Result<ChunkIdentifier> {
        let chunks = data
            .chunks(self.chunk_size)
            .map(|chunk| (Identifier::new(chunk), chunk))
            .collect::<Vec<_>>();

        let futures = chunks
            .clone()
            .into_iter()
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .map(|(id, chunk)| async move {
                match self.provider.get_content_writer(&id).await {
                    Ok(mut writer) => {
                        match writer.write_all(chunk).await {
                            Ok(_) => {}
                            Err(err) => {
                                return Err(anyhow::anyhow!(
                                    "failed to write chunk for `{}`: {}",
                                    id,
                                    err
                                )
                                .into());
                            }
                        };

                        writer
                            .shutdown()
                            .await
                            .map_err(|err| {
                                anyhow::anyhow!("failed to shutdown writer for `{}`: {}", id, err)
                                    .into()
                            })
                            .map(|_| ())
                    }
                    Err(Error::AlreadyExists) => Ok(()),
                    Err(err) => Err(err),
                }
            });

        for futures_chunk in &futures.chunks(self.max_parallel_uploads) {
            futures::future::join_all(futures_chunk)
                .await
                .into_iter()
                .collect::<Result<Vec<_>>>()?;
        }

        let ids = chunks.into_iter().map(|(id, _)| id).collect::<Vec<_>>();

        // Heuristic to avoid reallocs: probably a bit wasteful but good enough.
        let mut buf = Vec::with_capacity(ids.len() * Identifier::SMALL_IDENTIFIER_SIZE);

        let chunk_index = ChunkIndex::Linear(ids);
        match chunk_index.write_all_to(&mut buf) {
            Ok(()) => self.provider.write_content(&buf).await.map(|id| {
                ChunkIdentifier::new(data.len().try_into().expect("data_size too large"), id)
            }),
            Err(err) => Err(anyhow::anyhow!("failed to write chunk index: {}", err).into()),
        }
    }
}

struct AsyncReadStore {
    state: AsyncReadStoreState,
    refs: usize,
    size: usize,
}
enum AsyncReadStoreState {
    Single(Option<ContentAsyncRead>),
    Multi(Option<Vec<u8>>),
}

impl AsyncReadStore {
    pub fn new(reader: ContentAsyncRead, size: usize) -> Self {
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
                    Ok(reader)
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum ChunkFormat {
    Linear = 1,
}

impl ChunkFormat {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Linear),
            _ => None,
        }
    }
}

enum ChunkIndex {
    Linear(Vec<Identifier>),
}

impl ChunkIndex {
    pub async fn read_from(mut r: impl AsyncRead + Unpin) -> Result<Self> {
        let mut buf = [0u8; 1];

        r.read_exact(&mut buf).await.map_err(|err| {
            Error::InvalidChunkIndex(anyhow::anyhow!("failed to read chunk format: {}", err))
        })?;

        match ChunkFormat::from_u8(buf[0]).ok_or_else(|| {
            Error::InvalidChunkIndex(anyhow::anyhow!("invalid chunk format: {}", buf[0]))
        })? {
            ChunkFormat::Linear => {
                // Now we must read a list of identifiers, each prefixed with their size.
                let mut identifiers = Vec::new();
                let mut id_buf = [0u8; 256];

                loop {
                    let id_size = match r.read_exact(&mut buf).await {
                        Ok(_) => buf[0] as usize,
                        Err(err) => {
                            if err.kind() == std::io::ErrorKind::UnexpectedEof {
                                break;
                            }

                            return Err(Error::InvalidChunkIndex(anyhow::anyhow!(
                                "failed to read chunk index: {}",
                                err
                            )));
                        }
                    };

                    r.read_exact(&mut id_buf[..id_size]).await.map_err(|err| {
                        Error::InvalidChunkIndex(anyhow::anyhow!(
                            "failed to read chunk index: {}",
                            err
                        ))
                    })?;

                    identifiers.push(Identifier::read_from(std::io::Cursor::new(
                        &id_buf[..id_size],
                    ))?);
                }

                Ok(Self::Linear(identifiers))
            }
        }
    }

    pub fn write_all_to(&self, mut w: impl Write) -> std::io::Result<()> {
        w.write_all(&[self.format() as u8])?;

        match self {
            Self::Linear(identifiers) => {
                for id in identifiers {
                    let id_size = id.bytes_len();
                    w.write_all(&[id_size as u8])?;
                    id.write_to(&mut w)?;
                }

                Ok(())
            }
        }
    }

    fn format(&self) -> ChunkFormat {
        match self {
            ChunkIndex::Linear(_) => ChunkFormat::Linear,
        }
    }

    fn identifiers(&self) -> &Vec<Identifier> {
        match self {
            ChunkIndex::Linear(ids) => ids,
        }
    }
}
