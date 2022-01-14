mod backend;
mod grpc_backend;
mod local_backend;
mod sql_backend;

pub use backend::*;
pub use grpc_backend::*;
pub use local_backend::*;
pub use sql_backend::*;

use crate::{blob_storage::BlobStorageUrl, Result};

/// Represents a source control index.
pub struct Index {
    backend: Box<dyn IndexBackend>,
}

impl Index {
    pub fn new(url: &str) -> Result<Self> {
        let backend = new_index_backend(url)?;

        Ok(Self { backend })
    }

    pub(crate) fn backend(&self) -> &dyn IndexBackend {
        &*self.backend
    }

    pub async fn create(&self) -> Result<BlobStorageUrl> {
        self.backend.create_index().await
    }

    pub async fn destroy(&self) -> Result<()> {
        self.backend.destroy_index().await
    }

    pub async fn exists(&self) -> Result<bool> {
        self.backend.index_exists().await
    }
}
