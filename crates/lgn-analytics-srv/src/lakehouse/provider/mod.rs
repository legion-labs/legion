mod connection;

use async_trait::async_trait;
pub use connection::DataLakeConnection;
use lgn_analytics::server::AnalyticsProvider;

use crate::log_entry::Searchable;
use crate::{call_tree::reduce_lod, metrics::MetricHandler};
use anyhow::{Context, Result};
use lgn_analytics::prelude::*;
use lgn_analytics::types::{
    BlockSpansReply, Level, MetricBlockData, MetricBlockManifest, MetricBlockManifestRequest,
    MetricBlockRequest, ProcessInstance, ProcessLogReply,
};
use lgn_telemetry::types::{BlockMetadata, Process, Stream};
use lgn_tracing::{async_span_scope, span_fn};
use std::sync::Arc;

pub struct DataLakeProvider {
    pub connection: DataLakeConnection,
}

impl DataLakeProvider {
    pub fn new(connection: DataLakeConnection) -> Self {
        Self { connection }
    }

    #[span_fn]
    fn get_metric_handler(&self) -> MetricHandler {
        MetricHandler::new(
            Arc::clone(&self.connection.data_lake_blobs),
            Arc::clone(&self.connection.cache),
            self.connection.pool.clone(),
        )
    }
}

#[async_trait]
impl AnalyticsProvider for DataLakeProvider {
    #[span_fn]
    async fn get_process(&self, process_id: &str) -> Result<Process> {
        let mut connection = self.connection.pool.acquire().await?;
        find_process(&mut connection, process_id).await
    }

    #[span_fn]
    async fn list_recent_processes(&self, parent_process_id: &str) -> Result<Vec<ProcessInstance>> {
        let mut connection = self.connection.pool.acquire().await?;
        list_recent_processes(&mut connection, Some(parent_process_id)).await
    }

    #[span_fn]
    async fn search_processes(&self, search: &str) -> Result<Vec<ProcessInstance>> {
        let mut connection = self.connection.pool.acquire().await?;
        search_processes(&mut connection, search).await
    }

    #[span_fn]
    async fn list_process_streams(&self, process_id: &str) -> Result<Vec<Stream>> {
        let mut connection = self.connection.pool.acquire().await?;
        find_process_streams(&mut connection, process_id).await
    }

    #[span_fn]
    async fn list_stream_blocks(
        &self,
        stream_id: &str,
    ) -> Result<Vec<lgn_telemetry::types::BlockMetadata>> {
        let mut connection = self.connection.pool.acquire().await?;
        find_stream_blocks(&mut connection, stream_id).await
    }

    #[span_fn]
    async fn compute_spans_lod(
        &self,
        process: &Process,
        stream: &Stream,
        block_id: &str,
        lod_id: u32,
    ) -> Result<BlockSpansReply> {
        let lod0_reply = self
            .connection
            .jit_lakehouse
            .get_thread_block(process, stream, block_id)
            .await?;
        if lod_id == 0 {
            return Ok(lod0_reply);
        }
        let lod0 = lod0_reply.lod.unwrap();
        let reduced = reduce_lod(&lod0, lod_id);
        Ok(BlockSpansReply {
            scopes: lod0_reply.scopes,
            lod: Some(reduced),
            block_id: block_id.to_owned(),
            begin_ms: lod0_reply.begin_ms,
            end_ms: lod0_reply.end_ms,
        })
    }

    #[span_fn]
    async fn block_spans(
        &self,
        process: &Process,
        stream: &Stream,
        block_id: &str,
        lod_id: u32,
    ) -> Result<BlockSpansReply> {
        async_span_scope!("AnalyticsService::block_spans");
        if lod_id == 0 {
            self.connection
                .jit_lakehouse
                .get_thread_block(process, stream, block_id)
                .await
        } else {
            let cache_item_name = format!("spans_{}_{}", block_id, lod_id);
            self.connection
                .cache
                .get_or_put(&cache_item_name, async {
                    self.compute_spans_lod(process, stream, block_id, lod_id)
                        .await
                })
                .await
        }
    }

    #[allow(clippy::cast_precision_loss)]
    #[span_fn]
    async fn process_log(
        &self,
        process: &Process,
        begin: u64,
        end: u64,
        search: &Option<String>,
        level_threshold: Option<Level>,
    ) -> Result<ProcessLogReply> {
        let mut connection = self.connection.pool.acquire().await?;
        let mut entries = vec![];
        let mut entry_index: u64 = 0;

        let needles = match search {
            Some(search) if !search.is_empty() => Some(
                search
                    .split(' ')
                    .filter_map(|part| {
                        if part.is_empty() {
                            None
                        } else {
                            Some(part.to_lowercase())
                        }
                    })
                    .collect::<Vec<String>>(),
            ),
            _ => None,
        };

        for stream in find_process_log_streams(&mut connection, &process.process_id)
            .await
            .with_context(|| "error in find_process_log_streams")?
        {
            for block in find_stream_blocks(&mut connection, &stream.stream_id)
                .await
                .with_context(|| "error in find_stream_blocks")?
            {
                if (entry_index + block.nb_objects as u64) < begin {
                    entry_index += block.nb_objects as u64;
                } else {
                    for_each_log_entry_in_block(
                        &mut connection,
                        self.connection.data_lake_blobs.clone(),
                        process,
                        &stream,
                        &block,
                        |log_entry| {
                            if entry_index >= end {
                                return false;
                            }

                            if entry_index >= begin {
                                let valid_content = needles
                                    .as_ref()
                                    .map_or(true, |needles| log_entry.matches(needles.as_ref()));

                                let valid_level = level_threshold.map_or(true, |level_threshold| {
                                    log_entry.matches(level_threshold)
                                });

                                if valid_content && valid_level {
                                    entries.push(log_entry);
                                    entry_index += 1;
                                }
                            } else {
                                entry_index += 1;
                            }

                            true
                        },
                    )
                    .await
                    .with_context(|| "error in for_each_log_entry_in_block")?;
                }
            }
        }

        Ok(ProcessLogReply {
            entries,
            begin,
            end: entry_index,
        })
    }

    #[span_fn]
    async fn nb_process_log_entries(&self, process_id: &str) -> Result<u64> {
        let mut connection = self.connection.pool.acquire().await?;
        let mut count: u64 = 0;
        for stream in find_process_log_streams(&mut connection, process_id).await? {
            for b in find_stream_blocks(&mut connection, &stream.stream_id).await? {
                count += b.nb_objects as u64;
            }
        }
        Ok(count)
    }

    #[span_fn]
    async fn list_process_children(&self, process_id: &str) -> Result<Vec<Process>> {
        let mut connection = self.connection.pool.acquire().await?;
        let children = fetch_child_processes(&mut connection, process_id).await?;
        Ok(children)
    }

    #[span_fn]
    async fn get_block_metric(&self, request: MetricBlockRequest) -> Result<MetricBlockData> {
        let metric_handler = self.get_metric_handler();
        metric_handler.get_block_lod_data(request).await
    }

    #[span_fn]
    async fn get_block_metric_manifest(
        &self,
        request: MetricBlockManifestRequest,
    ) -> Result<MetricBlockManifest> {
        let metric_handler = self.get_metric_handler();
        metric_handler
            .get_block_manifest(&request.process_id, &request.block_id, &request.stream_id)
            .await
    }

    #[span_fn]
    async fn list_process_blocks(&self, process_id: &str, tag: &str) -> Result<Vec<BlockMetadata>> {
        let mut connection = self.connection.pool.acquire().await?;
        let blocks = find_process_blocks(&mut connection, process_id, tag).await?;
        Ok(blocks)
    }

    #[span_fn]
    #[allow(unused_variables)]
    async fn build_timeline_tables(&self, process_id: &str) -> Result<()> {
        #[cfg(feature = "deltalake-proto")]
        self.jit_lakehouse.build_timeline_tables(process_id).await?;
        Ok(())
    }
}
