use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait JitLakehouse: Send + Sync {
    async fn build_timeline_tables(&self, process_id: &str) -> Result<()>;
}
