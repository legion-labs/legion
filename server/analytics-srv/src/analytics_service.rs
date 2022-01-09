use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;
use async_recursion::async_recursion;
use lgn_analytics::prelude::*;
use lgn_telemetry_proto::analytics::performance_analytics_server::PerformanceAnalytics;
use lgn_telemetry_proto::analytics::BlockSpansReply;
use lgn_telemetry_proto::analytics::BlockSpansRequest;
use lgn_telemetry_proto::analytics::CallTree;
use lgn_telemetry_proto::analytics::CumulativeCallGraphReply;
use lgn_telemetry_proto::analytics::FetchProcessMetricRequest;
use lgn_telemetry_proto::analytics::FindProcessReply;
use lgn_telemetry_proto::analytics::FindProcessRequest;
use lgn_telemetry_proto::analytics::ListProcessChildrenRequest;
use lgn_telemetry_proto::analytics::ListProcessMetricsRequest;
use lgn_telemetry_proto::analytics::ListProcessStreamsRequest;
use lgn_telemetry_proto::analytics::ListStreamBlocksReply;
use lgn_telemetry_proto::analytics::ListStreamBlocksRequest;
use lgn_telemetry_proto::analytics::ListStreamsReply;
use lgn_telemetry_proto::analytics::LogEntry;
use lgn_telemetry_proto::analytics::ProcessChildrenReply;
use lgn_telemetry_proto::analytics::ProcessCumulativeCallGraphRequest;
use lgn_telemetry_proto::analytics::ProcessListReply;
use lgn_telemetry_proto::analytics::ProcessLogReply;
use lgn_telemetry_proto::analytics::ProcessLogRequest;
use lgn_telemetry_proto::analytics::ProcessMetricReply;
use lgn_telemetry_proto::analytics::ProcessMetricsReply;
use lgn_telemetry_proto::analytics::ProcessNbLogEntriesReply;
use lgn_telemetry_proto::analytics::ProcessNbLogEntriesRequest;
use lgn_telemetry_proto::analytics::RecentProcessesRequest;
use lgn_telemetry_proto::analytics::SearchProcessRequest;
use lgn_tracing::dispatch::init_thread_stream;
use lgn_tracing::prelude::*;
use tonic::{Request, Response, Status};

use crate::cache::DiskCache;
use crate::call_tree::compute_block_call_tree;
use crate::call_tree::compute_block_spans;
use crate::call_tree::reduce_lod;
use crate::cumulative_call_graph::compute_cumulative_call_graph;
use crate::metrics;

static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);

struct RequestGuard {
    begin_ticks: i64,
}

impl RequestGuard {
    fn new() -> Self {
        init_thread_stream();
        let previous_count = REQUEST_COUNT.fetch_add(1, Ordering::SeqCst);
        metric_int!("Request Count", "count", previous_count);

        let begin_ticks = lgn_tracing::now();
        Self { begin_ticks }
    }
}

impl Drop for RequestGuard {
    fn drop(&mut self) {
        let end_ticks = lgn_tracing::now();
        let duration = end_ticks - self.begin_ticks;
        metric_int!("Request Duration", "ticks", duration as u64);
    }
}

pub struct AnalyticsService {
    pool: sqlx::any::AnyPool,
    data_dir: PathBuf,
    cache: DiskCache,
}

impl AnalyticsService {
    pub async fn new(pool: sqlx::AnyPool, data_dir: PathBuf) -> Result<Self> {
        Ok(Self {
            pool,
            data_dir,
            cache: DiskCache::new().await?,
        })
    }

    async fn find_process_impl(&self, process_id: &str) -> Result<lgn_telemetry_sink::ProcessInfo> {
        let mut connection = self.pool.acquire().await?;
        find_process(&mut connection, process_id).await
    }

    async fn list_recent_processes_impl(
        &self,
    ) -> Result<Vec<lgn_telemetry_proto::analytics::ProcessInstance>> {
        let mut connection = self.pool.acquire().await?;
        fetch_recent_processes(&mut connection).await
    }

    async fn search_processes_impl(
        &self,
        search: &str,
    ) -> Result<Vec<lgn_telemetry_proto::analytics::ProcessInstance>> {
        let mut connection = self.pool.acquire().await?;
        search_processes(&mut connection, search).await
    }

    async fn list_process_streams_impl(
        &self,
        process_id: &str,
    ) -> Result<Vec<lgn_telemetry_sink::StreamInfo>> {
        let mut connection = self.pool.acquire().await?;
        find_process_streams(&mut connection, process_id).await
    }

    async fn list_stream_blocks_impl(
        &self,
        stream_id: &str,
    ) -> Result<Vec<lgn_telemetry_sink::EncodedBlock>> {
        let mut connection = self.pool.acquire().await?;
        find_stream_blocks(&mut connection, stream_id).await
    }

    async fn get_call_tree(
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

    async fn compute_spans_lod(
        &self,
        process: &lgn_telemetry_sink::ProcessInfo,
        stream: &lgn_telemetry_sink::StreamInfo,
        block_id: &str,
        lod_id: u32,
    ) -> Result<BlockSpansReply> {
        if lod_id == 0 {
            let tree = self.get_call_tree(process, stream, block_id).await?;
            return compute_block_spans(tree, block_id);
        }
        let lod0_reply = self.block_spans_impl(process, stream, block_id, 0).await?;
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

    #[async_recursion]
    async fn block_spans_impl(
        &self,
        process: &lgn_telemetry_sink::ProcessInfo,
        stream: &lgn_telemetry_sink::StreamInfo,
        block_id: &str,
        lod_id: u32,
    ) -> Result<BlockSpansReply> {
        let cache_item_name = format!("spans_{}_{}", block_id, lod_id);
        self.cache
            .get_or_put(&cache_item_name, async {
                self.compute_spans_lod(process, stream, block_id, lod_id)
                    .await
            })
            .await
    }

    async fn process_cumulative_call_graph_impl(
        &self,
        process: &lgn_telemetry_sink::ProcessInfo,
        begin_ms: f64,
        end_ms: f64,
    ) -> Result<CumulativeCallGraphReply> {
        let mut connection = self.pool.acquire().await?;
        compute_cumulative_call_graph(&mut connection, &self.data_dir, process, begin_ms, end_ms)
            .await
    }

    #[allow(clippy::cast_precision_loss)]
    async fn process_log_impl(
        &self,
        process: &lgn_telemetry_sink::ProcessInfo,
        begin: u64,
        end: u64,
    ) -> Result<ProcessLogReply> {
        let mut connection = self.pool.acquire().await?;
        let mut entries = vec![];
        let inv_tsc_frequency = 1000.0 / process.tsc_frequency as f64; // factor out
        let ts_offset = process.start_ticks;
        let mut entry_index: u64 = 0;
        for stream in find_process_log_streams(&mut connection, &process.process_id).await? {
            for block in find_stream_blocks(&mut connection, &stream.stream_id).await? {
                if (entry_index + block.nb_objects as u64) < begin {
                    entry_index += block.nb_objects as u64;
                } else {
                    for_each_log_entry_in_block(
                        &mut connection,
                        &self.data_dir,
                        &stream,
                        &block,
                        |ts, entry| {
                            if entry_index >= end {
                                return false;
                            }
                            if entry_index >= begin {
                                let time_ms = (ts - ts_offset) as f64 * inv_tsc_frequency;
                                entries.push(LogEntry {
                                    msg: entry,
                                    time_ms,
                                });
                            }
                            entry_index += 1;

                            true
                        },
                    )
                    .await?;
                }
            }
        }

        Ok(ProcessLogReply {
            entries,
            begin,
            end: entry_index,
        })
    }

    async fn nb_process_log_entries_impl(
        &self,
        process_id: &str,
    ) -> Result<ProcessNbLogEntriesReply> {
        let mut connection = self.pool.acquire().await?;
        let mut count: u64 = 0;
        for stream in find_process_log_streams(&mut connection, process_id).await? {
            for b in find_stream_blocks(&mut connection, &stream.stream_id).await? {
                count += b.nb_objects as u64;
            }
        }
        Ok(ProcessNbLogEntriesReply { count })
    }

    async fn list_process_children_impl(&self, process_id: &str) -> Result<ProcessChildrenReply> {
        let mut connection = self.pool.acquire().await?;
        let children = fetch_child_processes(&mut connection, process_id).await?;
        Ok(ProcessChildrenReply {
            processes: children,
        })
    }

    async fn list_process_metrics_impl(&self, process_id: &str) -> Result<ProcessMetricsReply> {
        let mut connection = self.pool.acquire().await?;
        let m = metrics::list_process_metrics(&mut connection, &self.data_dir, process_id).await?;
        let time_range =
            metrics::get_process_metrics_time_range(&mut connection, process_id).await?;
        Ok(ProcessMetricsReply {
            metrics: m,
            min_time_ms: time_range.0,
            max_time_ms: time_range.1,
        })
    }

    async fn fetch_process_metric_impl(
        &self,
        process_id: &str,
        metric_name: &str,
        unit: &str,
        begin_ms: f64,
        end_ms: f64,
    ) -> Result<ProcessMetricReply> {
        let mut connection = self.pool.acquire().await?;
        Ok(ProcessMetricReply {
            points: metrics::fetch_process_metric(
                &mut connection,
                &self.data_dir,
                process_id,
                metric_name,
                unit,
                begin_ms,
                end_ms,
            )
            .await?,
        })
    }
}

#[tonic::async_trait]
impl PerformanceAnalytics for AnalyticsService {
    async fn find_process(
        &self,
        request: Request<FindProcessRequest>,
    ) -> Result<Response<FindProcessReply>, Status> {
        let _guard = RequestGuard::new();
        info!("find_process");
        let find_request = request.into_inner();
        match self.find_process_impl(&find_request.process_id).await {
            Ok(process) => {
                let reply = FindProcessReply {
                    process: Some(process),
                };
                Ok(Response::new(reply))
            }
            Err(e) => {
                return Err(Status::internal(format!("Error in find_process: {}", e)));
            }
        }
    }

    async fn list_recent_processes(
        &self,
        _request: Request<RecentProcessesRequest>,
    ) -> Result<Response<ProcessListReply>, Status> {
        let _guard = RequestGuard::new();
        info!("list_recent_processes");
        match self.list_recent_processes_impl().await {
            Ok(processes) => {
                let reply = ProcessListReply { processes };
                info!("list_recent_processes_impl ok");
                Ok(Response::new(reply))
            }
            Err(e) => {
                return Err(Status::internal(format!(
                    "Error in list_recent_processes_impl: {}",
                    e
                )));
            }
        }
    }

    async fn search_processes(
        &self,
        request: Request<SearchProcessRequest>,
    ) -> Result<Response<ProcessListReply>, Status> {
        let _guard = RequestGuard::new();
        info!("search_processes");
        let inner = request.into_inner();
        debug!("{}", &inner.search);
        match self.search_processes_impl(&inner.search).await {
            Ok(processes) => {
                let reply = ProcessListReply { processes };
                info!("list_recent_processes_impl ok");
                Ok(Response::new(reply))
            }
            Err(e) => {
                return Err(Status::internal(format!(
                    "Error in list_recent_processes_impl: {}",
                    e
                )));
            }
        }
    }

    async fn list_process_streams(
        &self,
        request: Request<ListProcessStreamsRequest>,
    ) -> Result<Response<ListStreamsReply>, Status> {
        let _guard = RequestGuard::new();
        info!("list_process_streams");
        let list_request = request.into_inner();
        match self
            .list_process_streams_impl(&list_request.process_id)
            .await
        {
            Ok(streams) => {
                let reply = ListStreamsReply { streams };
                info!("list_process_streams ok");
                Ok(Response::new(reply))
            }
            Err(e) => {
                return Err(Status::internal(format!(
                    "Error in list_process_streams: {}",
                    e
                )));
            }
        }
    }

    async fn list_stream_blocks(
        &self,
        request: Request<ListStreamBlocksRequest>,
    ) -> Result<Response<ListStreamBlocksReply>, Status> {
        let _guard = RequestGuard::new();
        let list_request = request.into_inner();
        match self.list_stream_blocks_impl(&list_request.stream_id).await {
            Ok(blocks) => {
                let reply = ListStreamBlocksReply { blocks };
                Ok(Response::new(reply))
            }
            Err(e) => {
                return Err(Status::internal(format!(
                    "Error in list_stream_blocks: {}",
                    e
                )));
            }
        }
    }

    async fn block_spans(
        &self,
        request: Request<BlockSpansRequest>,
    ) -> Result<Response<BlockSpansReply>, Status> {
        let _guard = RequestGuard::new();
        let inner_request = request.into_inner();
        if inner_request.process.is_none() {
            return Err(Status::internal(String::from(
                "Missing process in block_spans",
            )));
        }
        if inner_request.stream.is_none() {
            return Err(Status::internal(String::from(
                "Missing stream in block_spans",
            )));
        }

        match self
            .block_spans_impl(
                &inner_request.process.unwrap(),
                &inner_request.stream.unwrap(),
                &inner_request.block_id,
                inner_request.lod_id,
            )
            .await
        {
            Ok(block_spans) => Ok(Response::new(block_spans)),
            Err(e) => {
                return Err(Status::internal(format!("Error in block_spans: {}", e)));
            }
        }
    }

    async fn process_cumulative_call_graph(
        &self,
        request: Request<ProcessCumulativeCallGraphRequest>,
    ) -> Result<Response<CumulativeCallGraphReply>, Status> {
        let _guard = RequestGuard::new();
        let inner_request = request.into_inner();
        if inner_request.process.is_none() {
            return Err(Status::internal(String::from(
                "Missing process in process_cumulative_call_graph",
            )));
        }
        match self
            .process_cumulative_call_graph_impl(
                &inner_request.process.unwrap(),
                inner_request.begin_ms,
                inner_request.end_ms,
            )
            .await
        {
            Ok(reply) => Ok(Response::new(reply)),
            Err(e) => Err(Status::internal(format!(
                "Error in process_cumulative_call_graph: {}",
                e
            ))),
        }
    }

    async fn list_process_log_entries(
        &self,
        request: Request<ProcessLogRequest>,
    ) -> Result<Response<ProcessLogReply>, Status> {
        let _guard = RequestGuard::new();
        let inner_request = request.into_inner();
        if inner_request.process.is_none() {
            return Err(Status::internal(String::from(
                "Missing process in list_process_log_entries",
            )));
        }
        match self
            .process_log_impl(
                &inner_request.process.unwrap(),
                inner_request.begin,
                inner_request.end,
            )
            .await
        {
            Ok(reply) => Ok(Response::new(reply)),
            Err(e) => Err(Status::internal(format!(
                "Error in list_process_log_entries: {}",
                e
            ))),
        }
    }

    async fn nb_process_log_entries(
        &self,
        request: Request<ProcessNbLogEntriesRequest>,
    ) -> Result<Response<ProcessNbLogEntriesReply>, Status> {
        let _guard = RequestGuard::new();
        let inner_request = request.into_inner();
        if inner_request.process_id.is_empty() {
            return Err(Status::internal(String::from(
                "Missing process_id in nb_process_log_entries",
            )));
        }
        match self
            .nb_process_log_entries_impl(&inner_request.process_id)
            .await
        {
            Ok(reply) => Ok(Response::new(reply)),
            Err(e) => Err(Status::internal(format!(
                "Error in nb_process_log_entries: {}",
                e
            ))),
        }
    }

    async fn list_process_children(
        &self,
        request: Request<ListProcessChildrenRequest>,
    ) -> Result<Response<ProcessChildrenReply>, Status> {
        let _guard = RequestGuard::new();
        let inner_request = request.into_inner();
        if inner_request.process_id.is_empty() {
            return Err(Status::internal(String::from(
                "Missing process_id in list_process_children",
            )));
        }
        match self
            .list_process_children_impl(&inner_request.process_id)
            .await
        {
            Ok(reply) => Ok(Response::new(reply)),
            Err(e) => Err(Status::internal(format!(
                "Error in list_process_children: {}",
                e
            ))),
        }
    }

    async fn list_process_metrics(
        &self,
        request: Request<ListProcessMetricsRequest>,
    ) -> Result<Response<ProcessMetricsReply>, Status> {
        let _guard = RequestGuard::new();
        let inner_request = request.into_inner();
        if inner_request.process_id.is_empty() {
            return Err(Status::internal(String::from(
                "Missing process_id in list_process_metrics",
            )));
        }
        match self
            .list_process_metrics_impl(&inner_request.process_id)
            .await
        {
            Ok(reply) => Ok(Response::new(reply)),
            Err(e) => Err(Status::internal(format!(
                "Error in list_process_metrics: {}",
                e
            ))),
        }
    }

    async fn fetch_process_metric(
        &self,
        request: Request<FetchProcessMetricRequest>,
    ) -> Result<Response<ProcessMetricReply>, Status> {
        let _guard = RequestGuard::new();
        let inner_request = request.into_inner();
        if inner_request.process_id.is_empty() {
            return Err(Status::internal(String::from(
                "Missing process_id in fetch_process_metric",
            )));
        }
        match self
            .fetch_process_metric_impl(
                &inner_request.process_id,
                &inner_request.metric_name,
                &inner_request.unit,
                inner_request.begin_ms,
                inner_request.end_ms,
            )
            .await
        {
            Ok(reply) => Ok(Response::new(reply)),
            Err(e) => Err(Status::internal(format!(
                "Error in fetch_process_metric: {}",
                e
            ))),
        }
    }
}
