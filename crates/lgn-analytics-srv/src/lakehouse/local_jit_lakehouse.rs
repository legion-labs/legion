use std::{path::PathBuf, sync::Arc};

use crate::{
    call_tree::{compute_block_spans, process_thread_block},
    lakehouse::{
        jit_lakehouse::JitLakehouse,
        span_table_partition::{make_rows_from_tree, SpanRowGroup},
    },
};
use crate::{lakehouse::span_table::update_spans_delta_table, scope::ScopeHashMap};
use anyhow::{Context, Result};
use async_trait::async_trait;
use datafusion::arrow::{self, array::PrimitiveArray};
use datafusion::prelude::*;
use lgn_analytics::{prelude::*, time::ConvertTicks};
use lgn_blob_storage::BlobStorage;
use lgn_telemetry_proto::analytics::{
    BlockSpansReply, CallTree, ScopeDesc, Span, SpanBlockLod, SpanTrack,
};
use lgn_tracing::prelude::*;
use tokio::fs;

use super::{scope_table::write_scopes_parquet, span_table_partition::write_spans_parquet};

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
        scopes_file_path: PathBuf,
    ) -> Result<BlockSpansReply> {
        info!("writing thread block {}", block_id);
        if let Some(parent) = spans_file_path.parent() {
            tokio::fs::create_dir_all(&parent)
                .await
                .with_context(|| format!("creating directory {}", parent.display()))?;
        }
        if let Some(parent) = scopes_file_path.parent() {
            tokio::fs::create_dir_all(&parent)
                .await
                .with_context(|| format!("creating directory {}", parent.display()))?;
        }

        let convert_ticks = ConvertTicks::new(process);
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
        write_spans_parquet(&rows, &spans_file_path)?;

        write_scopes_parquet(&processed.scopes, &scopes_file_path)?;

        //todo: do not iterate twice
        let tree = CallTree {
            scopes: processed.scopes,
            root: Some(root),
        };
        Ok(compute_block_spans(tree, block_id))
    }

    async fn read_thread_block(
        &self,
        block_id: &str,
        spans_file_uri: &str,
        scopes_file_uri: &str,
    ) -> Result<BlockSpansReply> {
        let ctx = SessionContext::new();
        ctx.register_parquet("spans", spans_file_uri, ParquetReadOptions::default())
            .await?;
        ctx.register_parquet("scopes", scopes_file_uri, ParquetReadOptions::default())
            .await?;
        let df_scopes = ctx
            .sql("SELECT hash, name, filename, line from scopes")
            .await?;
        let mut scopes = ScopeHashMap::new();
        for batch in df_scopes.collect().await? {
            let hashes = batch
                .column(0)
                .as_any()
                .downcast_ref::<PrimitiveArray<arrow::datatypes::Int32Type>>()
                .with_context(|| "casting hashes array")?;

            let names = batch
                .column(1)
                .as_any()
                .downcast_ref::<arrow::array::BinaryArray>()
                .with_context(|| "casting names array")?;

            let filenames = batch
                .column(2)
                .as_any()
                .downcast_ref::<arrow::array::BinaryArray>()
                .with_context(|| "casting filenames array")?;

            let lines = batch
                .column(3)
                .as_any()
                .downcast_ref::<PrimitiveArray<arrow::datatypes::Int32Type>>()
                .with_context(|| "casting lines array")?;

            for row in 0..batch.num_rows() {
                let hash = hashes.value(row) as u32;
                scopes.insert(
                    hash,
                    ScopeDesc {
                        name: String::from_utf8_lossy(names.value(row)).to_string(),
                        filename: String::from_utf8_lossy(filenames.value(row)).to_string(),
                        line: lines.value(row) as u32,
                        hash,
                    },
                );
            }
        }
        let span_batches = ctx
            .sql("select hash, depth, begin_ms, end_ms from spans")
            .await?
            .collect()
            .await?;
        let mut lod = SpanBlockLod {
            lod_id: 0,
            tracks: vec![],
        };
        for batch in span_batches {
            let hashes = batch
                .column(0)
                .as_any()
                .downcast_ref::<PrimitiveArray<arrow::datatypes::Int32Type>>()
                .with_context(|| "casting hashes array")?;
            let depths = batch
                .column(1)
                .as_any()
                .downcast_ref::<PrimitiveArray<arrow::datatypes::Int32Type>>()
                .with_context(|| "casting depths array")?;
            let begins = batch
                .column(2)
                .as_any()
                .downcast_ref::<PrimitiveArray<arrow::datatypes::Float64Type>>()
                .with_context(|| "casting begins array")?;
            let ends = batch
                .column(3)
                .as_any()
                .downcast_ref::<PrimitiveArray<arrow::datatypes::Float64Type>>()
                .with_context(|| "casting ends array")?;
            for row in 0..batch.num_rows() {
                let depth = depths.value(row) as u32;
                if lod.tracks.len() <= depth as usize {
                    lod.tracks.push(SpanTrack { spans: vec![] });
                }
                assert!(lod.tracks.len() > depth as usize);
                let span = Span {
                    scope_hash: hashes.value(row) as u32,
                    begin_ms: begins.value(row),
                    end_ms: ends.value(row),
                    alpha: 255,
                };
                lod.tracks[depth as usize].spans.push(span);
            }
        }
        let (min_begin, max_end) = if !lod.tracks.is_empty() && !lod.tracks[0].spans.is_empty() {
            (
                lod.tracks[0].spans[0].begin_ms,
                lod.tracks[0].spans[lod.tracks[0].spans.len() - 1].end_ms,
            )
        } else {
            (f64::MAX, f64::MIN)
        };
        Ok(BlockSpansReply {
            scopes,
            lod: Some(lod),
            block_id: block_id.to_owned(),
            begin_ms: min_begin,
            end_ms: max_end,
        })
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
            return self
                .write_thread_block(process, stream, block_id, spans_file_path, scopes_file_path)
                .await;
        }

        self.read_thread_block(
            block_id,
            &spans_file_path.to_string_lossy(),
            &scopes_file_path.to_string_lossy(),
        )
        .await
    }
}
