use legion_telemetry_proto::analytics::performance_analytics_server::PerformanceAnalytics;
use legion_telemetry_proto::analytics::ProcessListReply;
use legion_telemetry_proto::analytics::RecentProcessesRequest;
use std::path::PathBuf;
use tonic::{Request, Response, Status};

pub struct AnalyticsService {
    db_pool: sqlx::any::AnyPool,
    data_dir: PathBuf,
}

impl AnalyticsService {
    pub fn new(db_pool: sqlx::AnyPool, data_dir: PathBuf) -> Self {
        Self { db_pool, data_dir }
    }
}

#[tonic::async_trait]
impl PerformanceAnalytics for AnalyticsService {
    async fn list_recent_processes(
        &self,
        _request: Request<RecentProcessesRequest>,
    ) -> Result<Response<ProcessListReply>, Status> {
        return Err(Status::internal(String::from("not implemented")));
    }
}
