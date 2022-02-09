use anyhow::Result;
use lgn_blob_storage::BlobStorage;
use lgn_telemetry_proto::analytics::CallTree;
use std::sync::Arc;

use crate::{cache::DiskCache, call_tree::compute_block_call_tree};

pub struct CallTreeStore {
    pool: sqlx::any::AnyPool,
    blob_storage: Arc<dyn BlobStorage>,
    cache: DiskCache,
}

impl CallTreeStore {
    pub fn new(
        pool: sqlx::AnyPool,
        blob_storage: Arc<dyn BlobStorage>,
        cache_storage: Arc<dyn BlobStorage>,
    ) -> Self {
        Self {
            pool,
            blob_storage,
            cache: DiskCache::new(cache_storage),
        }
    }

    pub async fn get_call_tree(
        &self,
        process: &lgn_telemetry_sink::ProcessInfo,
        stream: &lgn_telemetry_sink::StreamInfo,
        block_id: &str,
    ) -> Result<CallTree> {
        let cache_item_name = format!("tree_{}", block_id);
        self.cache
            .get_or_put(&cache_item_name, async {
                let mut connection = self.pool.acquire().await?;
                compute_block_call_tree(
                    &mut connection,
                    self.blob_storage.clone(),
                    process,
                    stream,
                    block_id,
                )
                .await
            })
            .await
    }
}
