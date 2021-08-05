use crate::{http_repository_query::HTTPRepositoryQuery, sql::*, sql_repository_query::*, *};
use std::{path::PathBuf, sync::Arc};
use url::Url;

pub struct RepositoryConnection {
    blob_storage_spec: BlobStorageSpec,
    compressed_blob_cache: PathBuf,
    repo_query: Box<dyn RepositoryQuery + Send>,
}

impl RepositoryConnection {
    pub async fn new(repo_uri: &str, compressed_blob_cache: PathBuf) -> Result<Self, String> {
        let specified_uri = Url::parse(repo_uri).unwrap();
        let repo_query: Box<dyn RepositoryQuery + Send>;
        let mut url_path = String::from(specified_uri.path());
        let path = url_path.split_off(1); //remove leading /
        match specified_uri.scheme() {
            "lsc" => {
                let host = specified_uri.host().unwrap();
                let port = specified_uri.port().unwrap_or(80);
                let url = format!("http://{}:{}/lsc", host, port);
                repo_query = Box::new(HTTPRepositoryQuery::new(url, path)?);
            }
            "file" => {
                let db_url = format!("sqlite://{}/repo.db3", path);
                let pool = Arc::new(SqlConnectionPool::new(&db_url).await?);
                repo_query = Box::new(SqlRepositoryQuery::new(pool));
            }
            _ => {
                let pool = Arc::new(SqlConnectionPool::new(repo_uri).await?);
                repo_query = Box::new(SqlRepositoryQuery::new(pool));
            }
        }
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
    RepositoryConnection::new(&workspace.repo_uri, blob_cache_dir).await
}
