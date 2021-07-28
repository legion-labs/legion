use crate::{sql_repository_query::*, *};

pub struct RepositoryConnection {
    blob_store: Box<dyn BlobStorage>,
    repo_query: Box<dyn RepositoryQuery>,
}

impl RepositoryConnection {
    pub async fn new(
        repo_uri: &str,
        compressed_blob_cache: std::path::PathBuf,
    ) -> Result<Self, String> {
        let mut repo_query = Box::new(SqlRepositoryQuery::new(repo_uri)?);
        let blob_storage: Box<dyn BlobStorage> = match read_blob_storage_spec(repo_query.as_mut())?
        {
            BlobStorageSpec::LocalDirectory(blob_directory) => {
                Box::new(DiskBlobStorage { blob_directory })
            }
            BlobStorageSpec::S3Uri(s3uri) => {
                Box::new(S3BlobStorage::new(&s3uri, compressed_blob_cache).await?)
            }
        };

        Ok(Self {
            blob_store: blob_storage,
            repo_query,
        })
    }

    pub fn sql(&mut self) -> &mut sqlx::AnyConnection {
        self.repo_query.sql()
    }

    pub fn blob_storage(&self) -> &dyn BlobStorage {
        &*self.blob_store
    }
}

pub async fn connect_to_server(workspace: &Workspace) -> Result<RepositoryConnection, String> {
    let blob_cache_dir = std::path::Path::new(&workspace.root).join(".lsc/blob_cache");
    RepositoryConnection::new(&workspace.repo_uri, blob_cache_dir).await
}
