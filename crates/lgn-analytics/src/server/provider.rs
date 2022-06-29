use crate::types::{
    BlockSpansReply, Level, MetricBlockData, MetricBlockManifest, MetricBlockManifestRequest,
    MetricBlockRequest, ProcessInstance, ProcessLogReply,
};
use anyhow::Result;
use async_trait::async_trait;
use lgn_telemetry::types::{BlockMetadata, Process, Stream};

#[async_trait]
pub trait AnalyticsProvider {
    async fn get_process(&self, process_id: &str) -> Result<Process>;
    async fn list_recent_processes(&self, parent_process_id: &str) -> Result<Vec<ProcessInstance>>;
    async fn search_processes(&self, search: &str) -> Result<Vec<ProcessInstance>>;
    async fn list_process_streams(&self, process_id: &str) -> Result<Vec<Stream>>;
    async fn list_stream_blocks(
        &self,
        stream_id: &str,
    ) -> Result<Vec<lgn_telemetry::types::BlockMetadata>>;
    async fn compute_spans_lod(
        &self,
        process: &Process,
        stream: &Stream,
        block_id: &str,
        lod_id: u32,
    ) -> Result<BlockSpansReply>;
    async fn block_spans(
        &self,
        process: &Process,
        stream: &Stream,
        block_id: &str,
        lod_id: u32,
    ) -> Result<BlockSpansReply>;
    async fn process_log(
        &self,
        process: &Process,
        begin: u64,
        end: u64,
        search: &Option<String>,
        level_threshold: Option<Level>,
    ) -> Result<ProcessLogReply>;
    async fn nb_process_log_entries(&self, process_id: &str) -> Result<u64>;
    async fn list_process_children(&self, process_id: &str) -> Result<Vec<Process>>;
    async fn get_block_metric(&self, request: MetricBlockRequest) -> Result<MetricBlockData>;
    async fn get_block_metric_manifest(
        &self,
        request: MetricBlockManifestRequest,
    ) -> Result<MetricBlockManifest>;
    async fn list_process_blocks(&self, process_id: &str, tag: &str) -> Result<Vec<BlockMetadata>>;
    async fn build_timeline_tables(&self, process_id: &str) -> Result<()>;
}
