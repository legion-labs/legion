use crate::jit_lakehouse::JitLakehouse;
use anyhow::Result;
use async_trait::async_trait;
use lgn_analytics::prelude::*;
use lgn_tracing::prelude::*;

pub struct LocalJitLakehouse {
    pool: sqlx::any::AnyPool,
}

impl LocalJitLakehouse {
    pub fn new(pool: sqlx::any::AnyPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JitLakehouse for LocalJitLakehouse {
    async fn build_timeline_tables(&self, process_id: &str) -> Result<()> {
        let mut connection = self.pool.acquire().await?;
        let process = find_process(&mut connection, process_id).await?;
        warn!("build_timeline_tables {:?}", process);
        Ok(())
    }
}
