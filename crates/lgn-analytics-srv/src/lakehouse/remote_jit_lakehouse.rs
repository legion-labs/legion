use crate::lakehouse::jit_lakehouse::JitLakehouse;
use anyhow::Result;
use async_trait::async_trait;

pub struct RemoteJitLakehouse {}

#[async_trait]
impl JitLakehouse for RemoteJitLakehouse {
    async fn build_timeline_tables(&self, _process_id: &str) -> Result<()> {
        //not implemented
        Ok(())
    }
}
