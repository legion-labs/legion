use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    call_tree::{compute_block_spans, process_thread_block},
    lakehouse::{
        jit_lakehouse::JitLakehouse,
        span_table_partition::{make_rows_from_tree, SpanRowGroup, SpanTablePartitionLocalWriter},
    },
};
use crate::{lakehouse::span_table::update_spans_delta_table, scope::ScopeHashMap};
use anyhow::{Context, Result};
use async_trait::async_trait;
use lgn_analytics::{prelude::*, time::ConvertTicks};
use lgn_blob_storage::BlobStorage;
use lgn_telemetry_proto::analytics::{BlockSpansReply, CallTree, SpanBlockLod};
use lgn_tracing::prelude::*;
use tokio::fs;

pub struct LocalJitLakehouse {
    pool: sqlx::any::AnyPool,
    blob_storage: Arc<dyn BlobStorage>,
    tables_path: PathBuf,
}

impl LocalJitLakehouse {
    pub fn new(
        pool: sqlx::any::AnyPool,
        blob_storage: Arc<dyn BlobStorage>,
        tables_path: PathBuf,
    ) -> Self {
        Self {
            pool,
            blob_storage,
            tables_path,
        }
    }

    async fn write_thread_block(
        &self,
        process: &lgn_telemetry_sink::ProcessInfo,
        stream: &lgn_telemetry_sink::StreamInfo,
        block_id: &str,
        spans_file_path: PathBuf,
        _scopes_file_path: &Path,
    ) -> Result<BlockSpansReply> {
        if let Some(parent) = spans_file_path.parent() {
            tokio::fs::create_dir_all(&parent)
                .await
                .with_context(|| format!("creating directory {}", parent.display()))?;
        }

        let convert_ticks = ConvertTicks::new(&process);
        let processed = process_thread_block(
            self.pool.clone(),
            self.blob_storage.clone(),
            convert_ticks,
            stream,
            block_id,
        )
        .await?;
        if processed.call_tree_root.is_none() {
            return Ok(BlockSpansReply {
                scopes: ScopeHashMap::new(),
                lod: Some(SpanBlockLod {
                    lod_id: 0,
                    tracks: vec![],
                }),
                block_id: block_id.to_owned(),
                begin_ms: f64::MAX,
                end_ms: f64::MIN,
            });
        }
        let root = processed
            .call_tree_root
            .with_context(|| "reading root of call tree")?;
        let mut next_id = 1;
        let mut rows = SpanRowGroup::new();
        make_rows_from_tree(&root, &mut next_id, &mut rows);
        let mut writer = SpanTablePartitionLocalWriter::create(spans_file_path)?;
        writer.append(&rows)?;
        writer.close()?;

        //todo: do not iterate twice
        let tree = CallTree {
            scopes: processed.scopes.clone(),
            root: Some(root),
        };
        Ok(compute_block_spans(tree, &block_id))
    }
}

#[async_trait]
impl JitLakehouse for LocalJitLakehouse {
    async fn build_timeline_tables(&self, process_id: &str) -> Result<()> {
        async_span_scope!("build_timeline_tables");
        let mut connection = self.pool.acquire().await?;
        let process = find_process(&mut connection, process_id).await?;
        let convert_ticks = ConvertTicks::new(&process);
        let spans_table_path = self
            .tables_path
            .join("spans")
            .join(format!("process_id={}", process_id));
        fs::create_dir_all(&spans_table_path)
            .await
            .with_context(|| format!("creating folder {}", spans_table_path.display()))?;

        update_spans_delta_table(
            self.pool.clone(),
            self.blob_storage.clone(),
            process_id,
            &convert_ticks,
            spans_table_path,
        )
        .await?;
        Ok(())
    }

    async fn get_thread_block(
        &self,
        process: &lgn_telemetry_sink::ProcessInfo,
        stream: &lgn_telemetry_sink::StreamInfo,
        block_id: &str,
    ) -> Result<BlockSpansReply> {
        let spans_file_path = self
            .tables_path
            .join("spans")
            .join(format!("process_id={}", &process.process_id))
            .join(format!("block_id={}", block_id))
            .join("spans.parquet");

        let scopes_file_path = self
            .tables_path
            .join("scopes")
            .join(format!("process_id={}", &process.process_id))
            .join(format!("block_id={}", block_id))
            .join("scopes.parquet");

        if !spans_file_path.exists() || !scopes_file_path.exists() {
            let res = self
                .write_thread_block(
                    process,
                    stream,
                    block_id,
                    spans_file_path,
                    &scopes_file_path,
                )
                .await?;
            dbg!(res);
        }

        anyhow::bail!("not impl")
    }
}
