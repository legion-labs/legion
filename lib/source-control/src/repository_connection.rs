use anyhow::Result;
use std::sync::Arc;
use url::Url;

use crate::{
    http_repository_query::HttpRepositoryQuery,
    sql::SqlConnectionPool,
    sql_repository_query::{Databases, SqlRepositoryQuery},
    BlobStorage, RepositoryAddr, RepositoryQuery, Workspace,
};

pub struct RepositoryConnection {
    pub repo_query: Box<dyn RepositoryQuery>,
    pub blob_storage: Box<dyn BlobStorage>,
}

impl RepositoryConnection {
    async fn new(repo_addr: &RepositoryAddr) -> Result<Self> {
        let repo_query: Box<dyn RepositoryQuery>;

        match repo_addr {
            RepositoryAddr::Local(local_path) => {
                let sqlite_url = format!("sqlite://{}/repo.db3", local_path.display())
                    .parse()
                    .unwrap();
                let pool = Arc::new(SqlConnectionPool::new(sqlite_url).await?);
                repo_query = Box::new(SqlRepositoryQuery::new(pool, Databases::Sqlite));
            }
            RepositoryAddr::Remote(spec_uri) => {
                let url = Url::parse(spec_uri).unwrap();
                let path = String::from(url.path()).split_off(1); //remove leading /

                match url.scheme() {
                    "http" | "https" => {
                        repo_query = Box::new(HttpRepositoryQuery::new(url, path));
                    }
                    "mysql" => {
                        let pool =
                            Arc::new(SqlConnectionPool::new(spec_uri.parse().unwrap()).await?);
                        repo_query = Box::new(SqlRepositoryQuery::new(pool, Databases::Mysql));
                    }
                    unknown => {
                        anyhow::bail!("unknown remote url scheme {}", unknown);
                    }
                };
            }
        };

        let blob_storage_url = repo_query.read_blob_storage_spec().await?;
        let blob_storage = blob_storage_url.into_blob_storage().await?;

        Ok(Self {
            repo_query,
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
    RepositoryConnection::new(&workspace.repo_addr)
        .await
        .map(Arc::new)
}
