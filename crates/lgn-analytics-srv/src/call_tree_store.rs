use crate::{cache::DiskCache, call_tree::process_thread_block};
use anyhow::Result;
use lgn_analytics::prelude::*;
use lgn_analytics::time::ConvertTicks;
use lgn_blob_storage::BlobStorage;
use lgn_telemetry_proto::analytics::BlockAsyncData;
use lgn_telemetry_proto::analytics::CallTree;
use lgn_tracing::prelude::*;
use prost::Message;
use std::sync::Arc;

pub struct CallTreeStore {
    pool: sqlx::any::AnyPool,
    blob_storage: Arc<dyn BlobStorage>,
    cache: DiskCache,
}

impl CallTreeStore {
    #[span_fn]
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

    #[span_fn]
    async fn write_to_cache(
        &self,
        convert_ticks: ConvertTicks,
        stream: &lgn_telemetry_sink::StreamInfo,
        block_id: &str,
    ) -> Result<(CallTree, BlockAsyncData)> {
        let mut connection = self.pool.acquire().await?;
        let processed = process_thread_block(
            &mut connection,
            self.blob_storage.clone(),
            convert_ticks,
            stream,
            block_id,
        )
        .await?;
        let tree = CallTree {
            scopes: processed.scopes.clone(),
            root: processed.call_tree_root,
        };
        let async_data = BlockAsyncData {
            block_id: block_id.to_owned(),
            scopes: processed.scopes,
            events: processed.async_events,
        };
        let cache_async_name = format!("asyncblock_{}", block_id);
        let cache_tree_name = format!("tree_{}", block_id);
        let results = futures::join!(
            self.cache.put(&cache_tree_name, tree.encode_to_vec()),
            self.cache
                .put(&cache_async_name, async_data.encode_to_vec())
        );
        for r in [results.0, results.1] {
            if let Err(e) = r {
                error!("Error writing to cache: {}", e);
            }
        }
        Ok((tree, async_data))
    }

    #[span_fn]
    pub async fn get_call_tree(
        &self,
        convert_ticks: ConvertTicks,
        stream: &lgn_telemetry_sink::StreamInfo,
        block_id: &str,
    ) -> Result<CallTree> {
        let cache_tree_name = format!("tree_{}", block_id);
        if let Some(tree) = self.cache.get_cached_object(&cache_tree_name).await {
            return Ok(tree);
        }
        let (tree, _async_data) = self.write_to_cache(convert_ticks, stream, block_id).await?;
        Ok(tree)
    }

    #[span_fn]
    pub async fn get_block_async_data(&self, block_id: &str) -> Result<BlockAsyncData> {
        let cache_async_name = format!("asyncblock_{}", block_id);
        if let Some(async_data) = self.cache.get_cached_object(&cache_async_name).await {
            return Ok(async_data);
        }
        warn!("async data cache not found for block {}", block_id);
        let mut connection = self.pool.acquire().await?;
        let process = find_block_process(&mut connection, block_id).await?;
        let stream = find_block_stream(&mut connection, block_id).await?;
        let (_tree, async_data) = self
            .write_to_cache(ConvertTicks::new(&process), &stream, block_id)
            .await?;
        Ok(async_data)
    }
}
