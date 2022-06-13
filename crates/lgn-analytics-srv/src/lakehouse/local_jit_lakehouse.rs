use std::{path::PathBuf, sync::Arc};

#[cfg(feature = "deltalake-proto")]
use super::span_delta_table::update_spans_delta_table;

use super::span_table::{make_rows_from_tree, read_spans, write_spans_parquet, SpanRowGroup};
use crate::lakehouse::bytes_chunk_reader::BytesChunkReader;
use crate::scope::ScopeHashMap;
use crate::{
    call_tree::{compute_block_spans, process_thread_block},
    lakehouse::jit_lakehouse::JitLakehouse,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use lgn_analytics::time::ConvertTicks;
use lgn_blob_storage::BlobStorage;
use lgn_telemetry_proto::analytics::{BlockSpansReply, CallTree, ScopeDesc, SpanBlockLod};
use lgn_tracing::prelude::*;
use parquet::file::reader::FileReader;
use parquet::file::serialized_reader::SerializedFileReader;
use parquet::record::RowAccessor;
use tokio::io::AsyncReadExt;

use super::scope_table::write_scopes_parquet;

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
        write_spans_parquet(&rows, &spans_file_path).await?;

        write_scopes_parquet(&processed.scopes, &scopes_file_path).await?;

        //todo: do not iterate twice
        let tree = CallTree {
            scopes: processed.scopes,
            root: Some(root),
        };
        Ok(compute_block_spans(tree, block_id))
    }

    async fn read_spans_lod0(&self, spans_file_uri: &str) -> Result<SpanBlockLod> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut file = tokio::fs::File::open(spans_file_uri).await?;
        file.read_to_end(&mut buffer).await?;
        let bytes = BytesChunkReader {
            bytes: bytes::Bytes::from(buffer),
        };
        let file_reader = SerializedFileReader::new(bytes)?;
        read_spans(&file_reader)
    }

    async fn read_scopes(&self, spans_file_uri: &str) -> Result<ScopeHashMap> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut file = tokio::fs::File::open(spans_file_uri).await?;
        file.read_to_end(&mut buffer).await?;
        let bytes = BytesChunkReader {
            bytes: bytes::Bytes::from(buffer),
        };
        let file_reader = SerializedFileReader::new(bytes)?;
        let mut scopes = ScopeHashMap::new();
        for row in file_reader.get_row_iter(None)? {
            let hash = row.get_int(0)? as u32;
            let name = row.get_bytes(1)?;
            let filename = row.get_bytes(2)?;
            let line = row.get_int(3)? as u32;
            scopes.insert(
                hash,
                ScopeDesc {
                    name: String::from_utf8_lossy(name.data()).to_string(),
                    filename: String::from_utf8_lossy(filename.data()).to_string(),
                    line,
                    hash,
                },
            );
        }
        Ok(scopes)
    }

    async fn read_thread_block(
        &self,
        block_id: &str,
        spans_file_uri: &str,
        scopes_file_uri: &str,
    ) -> Result<BlockSpansReply> {
        let lod = self.read_spans_lod0(spans_file_uri).await?;
        let scopes = self.read_scopes(scopes_file_uri).await?;
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
    #[cfg(feature = "deltalake-proto")]
    async fn build_timeline_tables(&self, process_id: &str) -> Result<()> {
        async_span_scope!("build_timeline_tables");
        let mut connection = self.pool.acquire().await?;
        let process = lgn_analytics::find_process(&mut connection, process_id).await?;
        let convert_ticks = ConvertTicks::new(&process);
        let spans_table_path = self
            .tables_path
            .join("spans")
            .join(format!("process_id={}", process_id));
        tokio::fs::create_dir_all(&spans_table_path)
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
