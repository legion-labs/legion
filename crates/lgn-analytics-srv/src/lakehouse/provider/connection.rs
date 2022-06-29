use std::{path::PathBuf, str::FromStr, sync::Arc};

use anyhow::{Context, Result};
use lgn_blob_storage::{
    AwsS3BlobStorage, AwsS3Url, BlobStorage, LocalBlobStorage, Lz4BlobStorageAdapter,
};
use lgn_tracing::{flush_monitor::FlushMonitor, info, span_fn};

use crate::{
    cache::DiskCache,
    lakehouse::{
        jit_lakehouse::JitLakehouse, local_jit_lakehouse::LocalJitLakehouse,
        remote_jit_lakehouse::RemoteJitLakehouse,
    },
};

pub struct DataLakeConnection {
    pub pool: sqlx::any::AnyPool,
    pub data_lake_blobs: Arc<dyn BlobStorage>,
    pub cache: Arc<DiskCache>,
    pub jit_lakehouse: Arc<dyn JitLakehouse>,
}

impl DataLakeConnection {
    #[span_fn]
    pub fn new(
        pool: sqlx::any::AnyPool,
        data_lake_blobs: Arc<dyn BlobStorage>,
        cache_blobs: Arc<dyn BlobStorage>,
        jit_lakehouse: Arc<dyn JitLakehouse>,
    ) -> Self {
        Self {
            pool,
            data_lake_blobs,
            cache: Arc::new(DiskCache::new(cache_blobs)),
            jit_lakehouse,
        }
    }

    /// ``new_local`` serves a locally hosted data lake
    ///
    /// # Errors
    /// block storage must exist and sqlite database must accept connections
    #[span_fn]
    pub async fn new_local(
        data_lake_path: PathBuf,
        cache_path: PathBuf,
        lakehouse_uri: Option<String>,
    ) -> Result<Self> {
        let blocks_folder = data_lake_path.join("blobs");
        let data_lake_blobs = Arc::new(LocalBlobStorage::new(blocks_folder).await?);
        let cache_blobs = Arc::new(Lz4BlobStorageAdapter::new(
            LocalBlobStorage::new(cache_path.clone()).await?,
        ));
        let db_path = data_lake_path.join("telemetry.db3");
        let db_uri = format!("sqlite://{}", db_path.to_str().unwrap().replace('\\', "/"));
        let pool = sqlx::any::AnyPoolOptions::new()
            .connect(&db_uri)
            .await
            .with_context(|| String::from("Connecting to telemetry database"))?;
        let lakehouse = new_jit_lakehouse(
            lakehouse_uri
                .unwrap_or_else(|| cache_path.join("tables").to_string_lossy().to_string()),
            pool.clone(),
            data_lake_blobs.clone(),
        )
        .await?;
        Ok(Self::new(pool, data_lake_blobs, cache_blobs, lakehouse))
    }

    /// ``new_remote`` serves a remote data lake through mysql and s3
    ///
    /// # Errors
    /// block storage must exist and mysql database must accept connections
    #[span_fn]
    pub async fn new_remote(
        db_uri: &str,
        s3_url_data_lake: &str,
        s3_url_cache: String,
        lakehouse_uri: Option<String>,
    ) -> Result<Self> {
        info!("connecting to blob storage");
        let data_lake_blobs =
            Arc::new(AwsS3BlobStorage::new(AwsS3Url::from_str(s3_url_data_lake)?).await);
        let cache_blobs = Arc::new(Lz4BlobStorageAdapter::new(
            AwsS3BlobStorage::new(AwsS3Url::from_str(&s3_url_cache)?).await,
        ));
        let pool = sqlx::any::AnyPoolOptions::new()
            .max_connections(10)
            .connect(db_uri)
            .await
            .with_context(|| String::from("Connecting to telemetry database"))?;

        let lakehouse = new_jit_lakehouse(
            lakehouse_uri.unwrap_or_else(|| {
                if s3_url_cache.ends_with('/') {
                    s3_url_cache + "tables/"
                } else {
                    s3_url_cache + "/tables/"
                }
            }),
            pool.clone(),
            data_lake_blobs.clone(),
        )
        .await?;

        Ok(Self::new(pool, data_lake_blobs, cache_blobs, lakehouse))
    }
}

async fn new_jit_lakehouse(
    uri: String,
    pool: sqlx::AnyPool,
    data_lake_blobs: Arc<dyn BlobStorage>,
) -> Result<Arc<dyn JitLakehouse>> {
    if uri.starts_with("s3://") {
        Ok(Arc::new(
            RemoteJitLakehouse::new(pool, data_lake_blobs, AwsS3Url::from_str(&uri)?).await,
        ))
    } else {
        Ok(Arc::new(LocalJitLakehouse::new(
            pool,
            data_lake_blobs,
            PathBuf::from(uri),
        )))
    }
}
