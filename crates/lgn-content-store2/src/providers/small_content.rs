use async_trait::async_trait;
use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncWrite};

use crate::{ContentReader, ContentWriter, Error, Identifier, Result};

/// A `SmallContentProvider` is a provider that implements the small-content optimization or delegates to a specified provider.
pub struct SmallContentProvider<Inner> {
    inner: Inner,
    size_threshold: usize,
}

impl<Inner> SmallContentProvider<Inner> {
    /// Instanciate a new small content provider that wraps the specified
    /// provider using the default identifier size threshold.
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            size_threshold: Identifier::SIZE_THRESHOLD,
        }
    }

    /// Instanciate a new small content provider that wraps the specified
    /// provider and with the specified size threshold.
    pub fn new_with_size_threshold(inner: Inner, size_threshold: usize) -> Self {
        Self {
            inner,
            size_threshold,
        }
    }
}

#[async_trait]
impl<Inner: ContentReader + Send + Sync> ContentReader for SmallContentProvider<Inner> {
    async fn get_content_reader(&self, id: &Identifier) -> Result<Pin<Box<dyn AsyncRead + Send>>> {
        if let Identifier::Data(data) = id {
            Ok(Box::pin(std::io::Cursor::new(data.to_vec())))
        } else {
            self.inner.get_content_reader(id).await
        }
    }

    // As an optimization, we specialize this method because it's much faster to
    // just return the data directly than to go through the whole async reader
    // stuff.
    async fn read_content(&self, id: &Identifier) -> Result<Vec<u8>> {
        if let Identifier::Data(data) = id {
            Ok(data.to_vec())
        } else {
            self.inner.read_content(id).await
        }
    }
}

#[async_trait]
impl<Inner: ContentWriter + Send + Sync> ContentWriter for SmallContentProvider<Inner> {
    async fn get_content_writer(&self, id: &Identifier) -> Result<Pin<Box<dyn AsyncWrite + Send>>> {
        if id.is_data() {
            return Err(Error::AlreadyExists);
        }

        self.inner.get_content_writer(id).await
    }

    async fn write_content(&self, data: &[u8]) -> Result<Identifier> {
        if data.len() > self.size_threshold {
            self.inner.write_content(data).await
        } else {
            Ok(Identifier::Data(data.into()))
        }
    }
}
