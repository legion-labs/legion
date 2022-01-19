mod backend;
mod grpc_backend;
mod local_backend;
mod sql_backend;

pub use backend::*;
pub use grpc_backend::*;
pub use local_backend::*;
pub use sql_backend::*;

use crate::{BlobStorageUrl, Result};

/// Represents a source control index.
pub struct Index(Box<dyn IndexBackend>);

impl Index {
    pub fn new(url: &str) -> Result<Self> {
        let backend = new_index_backend(url)?;

        Ok(Self(backend))
    }

    pub async fn create(&self) -> Result<BlobStorageUrl> {
        self.0.create_index().await
    }

    pub async fn destroy(&self) -> Result<()> {
        self.0.destroy_index().await
    }

    pub async fn exists(&self) -> Result<bool> {
        self.0.index_exists().await
    }
}
