use anyhow::Result;
use lgn_telemetry_proto::analytics::CallTree;
use std::path::PathBuf;

use crate::{cache::DiskCache, call_tree::compute_block_call_tree};

pub struct CallTreeStore {
    pool: sqlx::any::AnyPool,
    data_dir: PathBuf,
    cache: DiskCache,
}

impl CallTreeStore {
    pub async fn new(pool: sqlx::AnyPool, data_dir: PathBuf) -> Result<Self> {
        Ok(Self {
            pool,
            data_dir,
            cache: DiskCache::new().await?,
        })
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
                compute_block_call_tree(&mut connection, &self.data_dir, process, stream, block_id)
                    .await
            })
            .await
    }
}
