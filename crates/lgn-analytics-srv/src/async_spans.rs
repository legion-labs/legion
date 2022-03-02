use anyhow::Result;
use lgn_telemetry_proto::analytics::BlockAsyncEventsStatReply;

pub fn compute_block_async_stats(
    _connection: &mut sqlx::AnyConnection,
    _process: &lgn_telemetry_proto::telemetry::Process,
) -> Result<BlockAsyncEventsStatReply> {
    anyhow::bail!("not implemented");
}
