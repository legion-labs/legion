use anyhow::Result;
use lgn_blob_storage::BlobStorage;
use std::sync::Arc;

use crate::{Index, IndexBackend, Workspace};

pub struct RepositoryConnection {
    pub index: Index,
    pub blob_storage: Box<dyn BlobStorage>,
}

impl RepositoryConnection {
    pub async fn new(index_url: &str) -> Result<Self> {
        let index = Index::new(index_url)?;
        let blob_storage = index
            .backend()
            .get_blob_storage_url()
            .await?
            .into_blob_storage()
            .await?;

        Ok(Self {
            index,
            blob_storage,
        })
    }

    pub fn index(&self) -> &Index {
        &self.index
    }

    pub fn index_backend(&self) -> &dyn IndexBackend {
        self.index.backend()
    }

    pub fn blob_storage(&self) -> &dyn BlobStorage {
        &*self.blob_storage
    }
}

pub async fn connect_to_server(workspace: &Workspace) -> Result<Arc<RepositoryConnection>> {
    RepositoryConnection::new(&workspace.index_url)
        .await
        .map(Arc::new)
}
