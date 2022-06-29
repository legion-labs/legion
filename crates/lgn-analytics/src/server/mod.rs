mod provider;
mod request_guard;

use lgn_online::server::{Error, Result};
use lgn_tracing::flush_monitor::FlushMonitor;
pub use provider::AnalyticsProvider;

use crate::api::analytics::server::{
    GetProcessRequest, GetProcessResponse, ListProcessStreamsRequest, ListProcessStreamsResponse,
    ListRecentProcessesRequest, ListRecentProcessesResponse, ListStreamBlocksRequest,
    ListStreamBlocksResponse, SearchProcessesRequest, SearchProcessesResponse,
};
use crate::api::analytics::Api;
use crate::types::CumulativeCallGraphComputedBlock;
use crate::types::Level;
use crate::types::MetricBlockData;
use crate::types::MetricBlockManifest;
use crate::types::MetricBlockManifestRequest;
use crate::types::MetricBlockRequest;
use crate::types::{
    CumulativeCallGraphBlockRequest, CumulativeCallGraphManifest,
    CumulativeCallGraphManifestRequest,
};
use async_trait::async_trait;
use lgn_tracing::prelude::*;
use request_guard::RequestGuard;

use std::sync::Arc;

pub struct Server {
    provider: Arc<dyn AnalyticsProvider + Send + Sync>,
    flush_monitor: FlushMonitor,
}

impl Server {
    pub fn new(provider: Arc<dyn AnalyticsProvider + Send + Sync>) -> Self {
        Self {
            provider,
            flush_monitor: FlushMonitor::default(),
        }
    }
}

#[async_trait]
impl Api for Server {
    async fn get_process(&self, request: GetProcessRequest) -> Result<GetProcessResponse> {
        self.flush_monitor.tick();
        async_span_scope!("AnalyticsService::get_process");
        let _guard = RequestGuard::new();
        info!("get_process");
        match self.provider.get_process(&request.process_id).await {
            Ok(process) => {
                info!("get_process ok");
                Ok(GetProcessResponse::Status200(process.into()))
            }
            Err(e) => {
                return Err(Error::internal(format!("Error in get_process: {}", e)));
            }
        }
    }

    async fn list_recent_processes(
        &self,
        request: ListRecentProcessesRequest,
    ) -> Result<ListRecentProcessesResponse> {
        self.flush_monitor.tick();
        async_span_scope!("AnalyticsService::list_recent_processes");
        let _guard = RequestGuard::new();
        info!("list_recent_processes");
        match self
            .provider
            .list_recent_processes(&request.parent_process_id)
            .await
        {
            Ok(processes) => {
                info!("list_recent_processes ok");
                Ok(ListRecentProcessesResponse::Status200(
                    processes.into_iter().map(Into::into).collect(),
                ))
            }
            Err(e) => {
                return Err(Error::internal(format!(
                    "Error in list_recent_processes: {}",
                    e
                )));
            }
        }
    }

    async fn search_processes(
        &self,
        request: SearchProcessesRequest,
    ) -> Result<SearchProcessesResponse> {
        self.flush_monitor.tick();
        async_span_scope!("AnalyticsService::search_processes");
        let _guard = RequestGuard::new();
        info!("search_processes");
        debug!("{}", &request.query);
        match self.provider.search_processes(&request.query).await {
            Ok(processes) => {
                info!("search_processes ok");
                Ok(SearchProcessesResponse::Status200(
                    processes.into_iter().map(Into::into).collect(),
                ))
            }
            Err(e) => {
                return Err(Error::internal(format!("Error in search_processes: {}", e)));
            }
        }
    }

    async fn list_process_streams(
        &self,
        request: ListProcessStreamsRequest,
    ) -> Result<ListProcessStreamsResponse> {
        self.flush_monitor.tick();
        async_span_scope!("AnalyticsService::list_process_streams");
        let _guard = RequestGuard::new();
        info!("list_process_streams");
        match self
            .provider
            .list_process_streams(&request.process_id)
            .await
        {
            Ok(streams) => {
                info!("list_process_streams ok");
                Ok(ListProcessStreamsResponse::Status200(
                    streams.into_iter().map(Into::into).collect(),
                ))
            }
            Err(e) => {
                return Err(Error::internal(format!(
                    "Error in list_process_streams: {}",
                    e
                )));
            }
        }
    }

    async fn list_stream_blocks(
        &self,
        request: ListStreamBlocksRequest,
    ) -> Result<ListStreamBlocksResponse> {
        self.flush_monitor.tick();
        async_span_scope!("AnalyticsService::list_stream_blocks");
        let _guard = RequestGuard::new();
        info!("list_stream_blocks");
        match self.provider.list_stream_blocks(&request.stream_id).await {
            Ok(blocks) => {
                info!("list_stream_blocks ok");
                Ok(ListStreamBlocksResponse::Status200(
                    blocks.into_iter().map(Into::into).collect(),
                ))
            }
            Err(e) => {
                return Err(Error::internal(format!(
                    "Error in list_stream_blocks: {}",
                    e
                )));
            }
        }
    }
}

// impl Server {
//     async fn block_spans(&self, request: BlockSpansRequest) -> Result<BlockSpansResponse> {
//         self.flush_monitor.tick();
//         async_span_scope!("AnalyticsService::block_spans");
//         let _guard = RequestGuard::new();
//         if request.process.is_none() {
//             error!("Missing process in block_spans");
//             return Err(Error::internal(String::from(
//                 "Missing process in block_spans",
//             )));
//         }
//         if request.stream.is_none() {
//             error!("Missing stream in block_spans");
//             return Err(Error::internal(String::from(
//                 "Missing stream in block_spans",
//             )));
//         }

//         match self
//             .provider
//             .block_spans(
//                 &request.process.unwrap(),
//                 &request.stream.unwrap(),
//                 &request.block_id,
//                 request.lod_id,
//             )
//             .await
//         {
//             Ok(block_spans) => Ok(block_spans),
//             Err(e) => {
//                 error!("Error in block_spans: {:?}", e);
//                 return Err(Error::internal(format!("Error in block_spans: {}", e)));
//             }
//         }
//     }

//     async fn get_cumulative_call_graph_manifest(
//         &self,
//         request: CumulativeCallGraphManifestRequest,
//     ) -> Result<CumulativeCallGraphManifest> {
//         self.flush_monitor.tick();
//         async_span_scope!("AnalyticsService::fetch_cumulative_call_graph_manifest");
//         let _guard = RequestGuard::new();
//         let handler = CumulativeCallGraphHandler::new(
//             self.connection.pool.clone(),
//             self.connection.jit_lakehouse.clone(),
//         );
//         match handler
//             .get_process_call_graph_manifest(request.process_id, request.begin_ms, request.end_ms)
//             .await
//         {
//             Ok(reply) => Ok(reply),
//             Err(e) => {
//                 error!("Error in fetch_cumulative_call_graph_manifest: {:?}", e);
//                 Err(Error::internal(format!(
//                     "Error in fetch_cumulative_call_graph_manifest: {}",
//                     e
//                 )))
//             }
//         }
//     }

//     async fn get_cumulative_call_graph_computed_block(
//         &self,
//         request: CumulativeCallGraphBlockRequest,
//     ) -> Result<CumulativeCallGraphComputedBlock> {
//         self.flush_monitor.tick();
//         async_span_scope!("AnalyticsService::fetch_cumulative_call_graph_computed_block");
//         let _guard = RequestGuard::new();
//         let handler = CumulativeCallGraphHandler::new(
//             self.connection.pool.clone(),
//             self.connection.jit_lakehouse.clone(),
//         );
//         match handler
//             .get_call_graph_computed_block(
//                 request.block_id,
//                 request.start_ticks,
//                 request.tsc_frequency,
//                 request.begin_ms,
//                 request.end_ms,
//             )
//             .await
//         {
//             Ok(reply) => Ok(reply),
//             Err(e) => {
//                 error!(
//                     "Error in fetch_cumulative_call_graph_computed_block: {:?}",
//                     e
//                 );
//                 Err(Error::internal(format!(
//                     "Error in fetch_cumulative_call_graph_computed_block: {}",
//                     e
//                 )))
//             }
//         }
//     }

//     async fn list_process_log_entries(
//         &self,
//         request: ProcessLogRequest,
//     ) -> Result<ProcessLogResponse> {
//         self.flush_monitor.tick();
//         async_span_scope!("AnalyticsService::list_process_log_entries");
//         let _guard = RequestGuard::new();
//         let process = match request.process {
//             Some(process) => process,
//             None => {
//                 error!("Missing process in list_process_log_entries");
//                 return Err(Error::internal(String::from(
//                     "Missing process in list_process_log_entries",
//                 )));
//             }
//         };

//         match self
//             .provider
//             .process_log(
//                 &process,
//                 request.begin,
//                 request.end,
//                 &request.search,
//                 request.level_threshold.and_then(Level::from_i32),
//             )
//             .await
//         {
//             Ok(reply) => Ok(reply),
//             Err(e) => {
//                 error!("Error in list_process_log_entries: {:?}", e);
//                 Err(Error::internal(format!(
//                     "Error in list_process_log_entries: {}",
//                     e
//                 )))
//             }
//         }
//     }

//     async fn nb_process_log_entries(
//         &self,
//         request: ProcessNbLogEntriesRequest,
//     ) -> Result<ProcessNbLogEntriesResponse> {
//         self.flush_monitor.tick();
//         async_span_scope!("AnalyticsService::nb_process_log_entries");
//         let _guard = RequestGuard::new();
//         if request.process_id.is_empty() {
//             error!("Missing process_id in nb_process_log_entries");
//             return Err(Error::internal(String::from(
//                 "Missing process_id in nb_process_log_entries",
//             )));
//         }
//         match self
//             .provider
//             .nb_process_log_entries(&request.process_id)
//             .await
//         {
//             Ok(reply) => Ok(reply),
//             Err(e) => {
//                 error!("Error in nb_process_log_entries: {:?}", e);
//                 Err(Error::internal(format!(
//                     "Error in nb_process_log_entries: {}",
//                     e
//                 )))
//             }
//         }
//     }

//     async fn list_process_children(
//         &self,
//         request: ListProcessChildrenRequest,
//     ) -> Result<ProcessChildrenResponse> {
//         self.flush_monitor.tick();
//         async_span_scope!("AnalyticsService::list_process_children");
//         let _guard = RequestGuard::new();
//         if request.process_id.is_empty() {
//             error!("Missing process_id in list_process_children");
//             return Err(Error::internal(String::from(
//                 "Missing process_id in list_process_children",
//             )));
//         }
//         match self
//             .provider
//             .list_process_children(&request.process_id)
//             .await
//         {
//             Ok(reply) => Ok(reply),
//             Err(e) => {
//                 error!("Error in list_process_children: {:?}", e);
//                 Err(Error::internal(format!(
//                     "Error in list_process_children: {}",
//                     e
//                 )))
//             }
//         }
//     }

//     async fn list_process_blocks(
//         &self,
//         request: ListProcessBlocksRequest,
//     ) -> Result<ProcessBlocksResponse> {
//         self.flush_monitor.tick();
//         async_span_scope!("AnalyticsService::list_process_blocks");
//         let _guard = RequestGuard::new();
//         match self
//             .provider
//             .list_process_blocks(&request.process_id, &request.tag)
//             .await
//         {
//             Ok(reply) => Ok(reply),
//             Err(e) => {
//                 error!("Error in list_process_blocks: {:?}", e);
//                 Err(Error::internal(format!(
//                     "Error in list_process_blocks: {}",
//                     e
//                 )))
//             }
//         }
//     }

//     async fn fetch_block_metric(&self, request: MetricBlockRequest) -> Result<MetricBlockData> {
//         self.flush_monitor.tick();
//         async_span_scope!("AnalyticsService::fetch_block_metric");
//         let _guard = RequestGuard::new();
//         match self.provider.fetch_block_metric(request).await {
//             Ok(reply) => Ok(reply),
//             Err(e) => {
//                 error!("Error in fetch_block_metric: {:?}", e);
//                 Err(Error::internal(format!(
//                     "Error in fetch_block_metric: {}",
//                     e
//                 )))
//             }
//         }
//     }

//     async fn fetch_block_metric_manifest(
//         &self,
//         request: MetricBlockManifestRequest,
//     ) -> Result<MetricBlockManifest> {
//         self.flush_monitor.tick();
//         async_span_scope!("AnalyticsService::fetch_block_metric_manifest");
//         let _guard = RequestGuard::new();
//         match self.provider.fetch_block_metric_manifest(request).await {
//             Ok(reply) => Ok(reply),
//             Err(e) => {
//                 error!("Error in fetch_block_metric_manifest: {:?}", e);
//                 Err(Error::internal(format!(
//                     "Error in fetch_block_metric_manifest: {}",
//                     e
//                 )))
//             }
//         }
//     }

//     #[span_fn]
//     async fn build_timeline_tables(
//         &self,
//         request: BuildTimelineTablesRequest,
//     ) -> Result<BuildTimelineTablesResponse> {
//         self.flush_monitor.tick();
//         async_span_scope!("AnalyticsService::build_timeline_tables");
//         let _guard = RequestGuard::new();
//         match self.provider.build_timeline_tables(request).await {
//             Ok(reply) => Ok(reply),
//             Err(e) => {
//                 error!("Error in build_timeline_tables: {:?}", e);
//                 Err(Error::internal(format!(
//                     "Error in build_timeline_tables: {}",
//                     e
//                 )))
//             }
//         }
//     }
// }
