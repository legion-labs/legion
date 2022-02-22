use async_trait::async_trait;

use crate::{
    traits::get_content_readers_impl, ContentAsyncRead, ContentAsyncWrite, ContentReader,
    ContentWriter, Error, Identifier, Result,
};

/// A `SmallContentProvider` is a provider that implements the small-content optimization or delegates to a specified provider.
pub struct SmallContentProvider<Inner> {
    inner: Inner,
}

impl<Inner> SmallContentProvider<Inner> {
    /// Instanciate a new small content provider that wraps the specified
    /// provider using the default identifier size threshold.
    pub fn new(inner: Inner) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<Inner: ContentReader + Send + Sync> ContentReader for SmallContentProvider<Inner> {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        if let Identifier::Data(data) = id {
            Ok(Box::pin(std::io::Cursor::new(data.to_vec())))
        } else {
            self.inner.get_content_reader(id).await
        }
    }

    async fn get_content_readers(
        &self,
        ids: &[Identifier],
    ) -> Result<Vec<Result<ContentAsyncRead>>> {
        get_content_readers_impl(self, ids).await
    }
}

#[async_trait]
impl<Inner: ContentWriter + Send + Sync> ContentWriter for SmallContentProvider<Inner> {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        if id.is_data() {
            return Err(Error::AlreadyExists);
        }

        self.inner.get_content_writer(id).await
    }
}
