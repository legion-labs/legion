use std::path::PathBuf;

use anyhow::Result;
use lgn_analytics::prelude::*;
use lgn_telemetry::prelude::*;
use lgn_telemetry_proto::analytics::performance_analytics_server::PerformanceAnalytics;
use lgn_telemetry_proto::analytics::BlockSpansReply;
use lgn_telemetry_proto::analytics::BlockSpansRequest;
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
use lgn_telemetry_proto::analytics::RecentProcessesRequest;
use lgn_telemetry_proto::analytics::SearchProcessRequest;
use std::sync::atomic::{AtomicU64, Ordering};
use tonic::{Request, Response, Status};

use crate::call_tree::compute_block_spans;
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
        static REQUEST_ID_METRIC: MetricDesc = MetricDesc {
            name: "Request ID",
            unit: "count",
        };
        record_int_metric(&REQUEST_ID_METRIC, previous_count);

        let begin_ticks = lgn_telemetry::now();
        Self { begin_ticks }
    }
}

impl Drop for RequestGuard {
    fn drop(&mut self) {
        let end_ticks = lgn_telemetry::now();
        let duration = end_ticks - self.begin_ticks;
        static REQUEST_TIME_METRIC: MetricDesc = MetricDesc {
            name: "Request Time",
            unit: "ticks",
        };
        record_int_metric(&REQUEST_TIME_METRIC, duration as u64);
    }
}

pub struct AnalyticsService {
    pool: sqlx::any::AnyPool,
    data_dir: PathBuf,
}

impl AnalyticsService {
    pub fn new(pool: sqlx::AnyPool, data_dir: PathBuf) -> Self {
        Self { pool, data_dir }
    }

    async fn find_process_impl(&self, process_id: &str) -> Result<lgn_telemetry::ProcessInfo> {
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
    ) -> Result<Vec<lgn_telemetry::StreamInfo>> {
        let mut connection = self.pool.acquire().await?;
        find_process_streams(&mut connection, process_id).await
    }

    async fn list_stream_blocks_impl(
        &self,
        stream_id: &str,
    ) -> Result<Vec<lgn_telemetry::EncodedBlock>> {
        let mut connection = self.pool.acquire().await?;
        find_stream_blocks(&mut connection, stream_id).await
    }

    async fn block_spans_impl(
        &self,
        process: &lgn_telemetry::ProcessInfo,
        stream: &lgn_telemetry::StreamInfo,
        block_id: &str,
    ) -> Result<BlockSpansReply> {
        let mut connection = self.pool.acquire().await?;
        compute_block_spans(&mut connection, &self.data_dir, process, stream, block_id).await
    }

    async fn process_cumulative_call_graph_impl(
        &self,
        process: &lgn_telemetry::ProcessInfo,
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
        process: &lgn_telemetry::ProcessInfo,
    ) -> Result<ProcessLogReply> {
        let mut connection = self.pool.acquire().await?;
        let mut entries = vec![];
        let inv_tsc_frequency = 1000.0 / process.tsc_frequency as f64; // factor out
        let ts_offset = process.start_ticks;
        for_each_process_log_entry(
            &mut connection,
            &self.data_dir,
            &process.process_id,
            |ts, entry| {
                let time_ms = (ts - ts_offset) as f64 * inv_tsc_frequency;
                entries.push(LogEntry {
                    msg: entry,
                    time_ms,
                });
            },
        )
        .await?;
        Ok(ProcessLogReply { entries })
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
        _process_id: &str,
        _metric_name: &str,
        _begin_ms: f64,
        _end_ms: f64,
    ) -> Result<ProcessMetricReply> {
        anyhow::bail!("not impl");
    }
}

#[tonic::async_trait]
impl PerformanceAnalytics for AnalyticsService {
    async fn find_process(
        &self,
        request: Request<FindProcessRequest>,
    ) -> Result<Response<FindProcessReply>, Status> {
        let _guard = RequestGuard::new();
        log::info!("find_process");
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
        log::info!("list_recent_processes");
        match self.list_recent_processes_impl().await {
            Ok(processes) => {
                let reply = ProcessListReply { processes };
                log::info!("list_recent_processes_impl ok");
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
        log::info!("search_processes");
        let inner = request.into_inner();
        dbg!(&inner.search);
        match self.search_processes_impl(&inner.search).await {
            Ok(processes) => {
                let reply = ProcessListReply { processes };
                log::info!("list_recent_processes_impl ok");
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
        log::info!("list_process_streams");
        let list_request = request.into_inner();
        match self
            .list_process_streams_impl(&list_request.process_id)
            .await
        {
            Ok(streams) => {
                let reply = ListStreamsReply { streams };
                log::info!("list_process_streams ok");
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
            )
            .await
        {
            Ok(block_spans) => Ok(Response::new(block_spans)),
            Err(e) => {
                return Err(Status::internal(format!("Error in block_call_tree: {}", e)));
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
        match self.process_log_impl(&inner_request.process.unwrap()).await {
            Ok(reply) => Ok(Response::new(reply)),
            Err(e) => Err(Status::internal(format!(
                "Error in list_process_log_entries: {}",
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
