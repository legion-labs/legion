use std::path::PathBuf;

use anyhow::Result;
use legion_analytics::prelude::*;
use legion_telemetry_proto::analytics::performance_analytics_server::PerformanceAnalytics;
use legion_telemetry_proto::analytics::BlockCallTreeReply;
use legion_telemetry_proto::analytics::BlockCallTreeRequest;
use legion_telemetry_proto::analytics::FindProcessReply;
use legion_telemetry_proto::analytics::FindProcessRequest;
use legion_telemetry_proto::analytics::ListProcessStreamsRequest;
use legion_telemetry_proto::analytics::ListStreamBlocksReply;
use legion_telemetry_proto::analytics::ListStreamBlocksRequest;
use legion_telemetry_proto::analytics::ListStreamsReply;
use legion_telemetry_proto::analytics::ProcessListReply;
use legion_telemetry_proto::analytics::RecentProcessesRequest;
use legion_telemetry_proto::analytics::ScopeInstance;

use tonic::{Request, Response, Status};

pub struct AnalyticsService {
    pool: sqlx::any::AnyPool,
    _data_dir: PathBuf,
}

impl AnalyticsService {
    pub fn new(pool: sqlx::AnyPool, data_dir: PathBuf) -> Self {
        Self {
            pool,
            _data_dir: data_dir,
        }
    }

    async fn find_process_impl(&self, process_id: &str) -> Result<legion_telemetry::ProcessInfo> {
        let mut connection = self.pool.acquire().await?;
        find_process(&mut connection, process_id).await
    }

    async fn list_recent_processes_impl(&self) -> Result<Vec<legion_telemetry::ProcessInfo>> {
        let mut connection = self.pool.acquire().await?;
        fetch_recent_processes(&mut connection).await
    }

    async fn list_process_streams_impl(
        &self,
        process_id: &str,
    ) -> Result<Vec<legion_telemetry::StreamInfo>> {
        let mut connection = self.pool.acquire().await?;
        find_process_streams(&mut connection, process_id).await
    }

    async fn list_stream_blocks_impl(
        &self,
        stream_id: &str,
    ) -> Result<Vec<legion_telemetry::EncodedBlock>> {
        let mut connection = self.pool.acquire().await?;
        find_stream_blocks(&mut connection, stream_id).await
    }

    async fn block_call_tree_impl(&self, _block_id: &str) -> Result<Vec<ScopeInstance>> {
        let mut _connection = self.pool.acquire().await?;
        anyhow::bail!("not impl")
    }
}

#[tonic::async_trait]
impl PerformanceAnalytics for AnalyticsService {
    async fn find_process(
        &self,
        request: Request<FindProcessRequest>,
    ) -> Result<Response<FindProcessReply>, Status> {
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

    async fn list_process_streams(
        &self,
        request: Request<ListProcessStreamsRequest>,
    ) -> Result<Response<ListStreamsReply>, Status> {
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

    async fn block_call_tree(
        &self,
        request: Request<BlockCallTreeRequest>,
    ) -> Result<Response<BlockCallTreeReply>, Status> {
        let inner_request = request.into_inner();
        match self.block_call_tree_impl(&inner_request.block_id).await {
            Ok(scopes) => {
                let reply = BlockCallTreeReply { scopes };
                Ok(Response::new(reply))
            }
            Err(e) => {
                return Err(Status::internal(format!("Error in block_call_tree: {}", e)));
            }
        }
    }
}
