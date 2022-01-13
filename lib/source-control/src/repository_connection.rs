use anyhow::Result;
use std::sync::Arc;

use crate::{blob_storage::BlobStorage, RepositoryQuery, RepositoryUrl, Workspace};

pub struct RepositoryConnection {
    pub repo_query: Box<dyn RepositoryQuery>,
    pub blob_storage: Box<dyn BlobStorage>,
}

impl RepositoryConnection {
    pub async fn new(url: RepositoryUrl) -> Result<Self> {
        let repository_query = url.into_query();
        let blob_storage = repository_query
            .get_blob_storage_url()
            .await?
            .into_blob_storage()
            .await?;

        Ok(Self {
            repo_query: repository_query,
            blob_storage,
        })
    }

    pub fn query(&self) -> &dyn RepositoryQuery {
        &*self.repo_query
    }

    pub fn blob_storage(&self) -> &dyn BlobStorage {
        &*self.blob_storage
    }
}

pub async fn connect_to_server(workspace: &Workspace) -> Result<Arc<RepositoryConnection>> {
    RepositoryConnection::new(workspace.repository_url.clone())
        .await
        .map(Arc::new)
}
