use std::path::PathBuf;

use anyhow::Result;
use legion_analytics::prelude::*;
use legion_telemetry_proto::analytics::performance_analytics_server::PerformanceAnalytics;
use legion_telemetry_proto::analytics::ProcessListReply;
use legion_telemetry_proto::analytics::RecentProcessesRequest;
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

    async fn list_recent_processes_impl(&self) -> Result<Vec<legion_telemetry::ProcessInfo>> {
        let mut connection = self.pool.acquire().await?;
        fetch_recent_processes(&mut connection).await
    }
}

#[tonic::async_trait]
impl PerformanceAnalytics for AnalyticsService {
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
}
