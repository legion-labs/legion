use anyhow::Result;
use legion_analytics::find_process_thread_streams;
use legion_telemetry_proto::analytics::CumulativeCallGraphReply;

pub(crate) async fn compute_cumulative_call_graph(
    connection: &mut sqlx::AnyConnection,
    process: &legion_telemetry::ProcessInfo,
) -> Result<CumulativeCallGraphReply> {
    let _streams = find_process_thread_streams(connection, &process.process_id).await?;
    anyhow::bail!("not impl")
}
