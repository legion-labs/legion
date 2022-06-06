use crate::lakehouse::jit_lakehouse::JitLakehouse;
use anyhow::Result;
use async_trait::async_trait;
use lgn_telemetry_proto::analytics::BlockSpansReply;

pub struct RemoteJitLakehouse {}

#[async_trait]
impl JitLakehouse for RemoteJitLakehouse {
    async fn build_timeline_tables(&self, _process_id: &str) -> Result<()> {
        //not implemented
        Ok(())
    }

    async fn get_thread_block(
        &self,
        _process: &lgn_telemetry_sink::ProcessInfo,
        _stream: &lgn_telemetry_sink::StreamInfo,
        _block_id: &str,
    ) -> Result<BlockSpansReply> {
        anyhow::bail!("not impl")
    }
}
