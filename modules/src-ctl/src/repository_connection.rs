use crate::*;
use futures::executor::block_on;
use sqlx::Connection;

pub struct RepositoryConnection {
    blob_store: Box<dyn BlobStorage>,
    sql_connection: sqlx::AnyConnection,
}

pub fn connect(database_uri: &str) -> Result<sqlx::AnyConnection, String> {
    match block_on(sqlx::AnyConnection::connect(database_uri)) {
        Ok(connection) => Ok(connection),
        Err(e) => Err(format!("Error connecting to {}: {}", database_uri, e)),
    }
}

#[derive(Debug)]
pub struct SqlConnectionPool {
    pub pool: sqlx::AnyPool,
}

pub fn make_sql_connection_pool(database_uri: &str) -> Result<SqlConnectionPool, String> {
    match block_on(
        sqlx::any::AnyPoolOptions::new()
            .max_connections(5)
            .connect(database_uri),
    ) {
        Ok(pool) => Ok(SqlConnectionPool { pool }),
        Err(e) => Err(format!("Error allocating database pool: {}", e)),
    }
}

impl RepositoryConnection {
    pub fn new(repo_uri: &str, compressed_blob_cache: std::path::PathBuf) -> Result<Self, String> {
        let mut c = connect(repo_uri)?;
        let blob_storage: Box<dyn BlobStorage> = match read_blob_storage_spec(&mut c)? {
            BlobStorageSpec::LocalDirectory(blob_directory) => {
                Box::new(DiskBlobStorage { blob_directory })
            }
            BlobStorageSpec::S3Uri(s3uri) => {
                Box::new(S3BlobStorage::new(&s3uri, compressed_blob_cache)?)
            }
        };

        Ok(Self {
            blob_store: blob_storage,
            sql_connection: c,
        })
    }

    pub fn sql(&mut self) -> &mut sqlx::AnyConnection {
        &mut self.sql_connection
    }

    pub fn blob_storage(&self) -> &dyn BlobStorage {
        &*self.blob_store
    }
}

pub fn connect_to_server(workspace: &Workspace) -> Result<RepositoryConnection, String> {
    let blob_cache_dir = std::path::Path::new(&workspace.root).join(".lsc/blob_cache");
    RepositoryConnection::new(&workspace.repo_uri, blob_cache_dir)
}
