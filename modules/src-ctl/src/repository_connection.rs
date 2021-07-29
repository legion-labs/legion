use crate::{sql::*, sql_repository_query::*, *};

pub struct RepositoryConnection {
    blob_store: Box<dyn BlobStorage>,
    repo_query: Box<dyn RepositoryQuery>,
    db_uri: String, //todo: remove
}

impl RepositoryConnection {
    pub async fn new(
        repo_uri: &str,
        compressed_blob_cache: std::path::PathBuf,
    ) -> Result<Self, String> {
        let repo_query = Box::new(SqlRepositoryQuery::new(repo_uri)?);
        let mut sql_connection = connect(repo_uri)?; //todo: remove
        let blob_storage: Box<dyn BlobStorage> = match read_blob_storage_spec(&mut sql_connection)?
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
            db_uri: String::from(repo_uri),
        })
    }

    pub fn sql(&self) -> sqlx::AnyConnection {
        connect(&self.db_uri).unwrap()
    }

    pub fn query(&self) -> &dyn RepositoryQuery {
        &*self.repo_query
    }

    pub fn blob_storage(&self) -> &dyn BlobStorage {
        &*self.blob_store
    }
}

pub async fn connect_to_server(workspace: &Workspace) -> Result<RepositoryConnection, String> {
    let blob_cache_dir = std::path::Path::new(&workspace.root).join(".lsc/blob_cache");
    RepositoryConnection::new(&workspace.repo_uri, blob_cache_dir).await
}
