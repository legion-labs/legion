use std::{path::PathBuf, sync::Arc};

use url::Url;

use crate::{
    http_repository_query::HTTPRepositoryQuery,
    sql::SqlConnectionPool,
    sql_repository_query::{Databases, SqlRepositoryQuery},
    BlobStorage, BlobStorageSpec, DiskBlobStorage, RepositoryAddr, RepositoryQuery, S3BlobStorage,
    Workspace,
};

pub struct RepositoryConnection {
    blob_storage_spec: BlobStorageSpec,
    compressed_blob_cache: PathBuf,
    repo_query: Box<dyn RepositoryQuery + Send>,
}

impl RepositoryConnection {
    pub async fn new(
        repo_addr: &RepositoryAddr,
        compressed_blob_cache: PathBuf,
    ) -> Result<Self, String> {
        let repo_query: Box<dyn RepositoryQuery + Send>;
        match repo_addr {
            RepositoryAddr::Local(local_path) => {
                let sqlite_url = format!("sqlite://{}/repo.db3", local_path.display());
                let pool = Arc::new(SqlConnectionPool::new(&sqlite_url).await?);
                repo_query = Box::new(SqlRepositoryQuery::new(pool, Databases::Sqlite));
            }
            RepositoryAddr::Remote(spec_uri) => {
                let uri = Url::parse(spec_uri).unwrap();
                let mut url_path = String::from(uri.path());
                let path = url_path.split_off(1); //remove leading /
                match uri.scheme() {
                    "lsc" => {
                        let host = uri.host().unwrap();
                        let port = uri.port().unwrap_or(80);
                        let url = format!("http://{}:{}/lsc", host, port);
                        repo_query = Box::new(HTTPRepositoryQuery::new(url, path)?);
                    }
                    "mysql" => {
                        let pool = Arc::new(SqlConnectionPool::new(spec_uri).await?);
                        repo_query = Box::new(SqlRepositoryQuery::new(pool, Databases::Mysql));
                    }
                    unknown => {
                        return Err(format!("unknown remote url scheme {}", unknown));
                    }
                };
            }
        };

        let blob_storage_spec = repo_query.read_blob_storage_spec().await?;
        Ok(Self {
            blob_storage_spec,
            compressed_blob_cache,
            repo_query,
        })
    }

    pub fn query(&self) -> &dyn RepositoryQuery {
        &*self.repo_query
    }

    pub async fn blob_storage(&self) -> Result<Box<dyn BlobStorage>, String> {
        match &self.blob_storage_spec {
            BlobStorageSpec::LocalDirectory(blob_directory) => Ok(Box::new(DiskBlobStorage {
                blob_directory: blob_directory.clone(),
            })),
            BlobStorageSpec::S3Uri(s3uri) => Ok(Box::new(
                S3BlobStorage::new(s3uri, self.compressed_blob_cache.clone()).await?,
            )),
        }
    }
}

pub async fn connect_to_server(workspace: &Workspace) -> Result<RepositoryConnection, String> {
    let blob_cache_dir = std::path::Path::new(&workspace.root).join(".lsc/blob_cache");
    RepositoryConnection::new(&workspace.repo_addr, blob_cache_dir).await
}
