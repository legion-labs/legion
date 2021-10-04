use analytics::*;
use anyhow::*;
use prost::Message;
use std::path::Path;
use transit::*;

pub async fn print_process_log(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    process_id: &str,
) -> Result<()> {
    for stream in find_process_log_streams(connection, process_id).await? {
        for b in find_stream_blocks(connection, &stream.stream_id).await? {
            let payload_path = data_path.join("blobs").join(&b.block_id);
            if !payload_path.exists() {
                bail!("payload binary file not found: {}", payload_path.display());
            }
            let buffer = std::fs::read(&payload_path)
                .with_context(|| format!("reading payload file {}", payload_path.display()))?;
            let payload = telemetry::telemetry_ingestion_proto::BlockPayload::decode(&*buffer)
                .with_context(|| format!("reading payload file {}", payload_path.display()))?;

            let dep_udts = stream
                .dependencies_metadata
                .as_ref()
                .unwrap()
                .as_transit_udt_vec();

            let dependencies = read_dependencies(&dep_udts, &payload.dependencies)?;
            let obj_udts = stream
                .objects_metadata
                .as_ref()
                .unwrap()
                .as_transit_udt_vec();
            parse_objects(&dependencies, &obj_udts, &payload.objects, |val| {
                if let Value::Object(obj) = val {
                    if obj.type_name == "LogMsgEvent" {
                        println!(
                            "[{}] {}",
                            obj.get::<u8>("level").unwrap(),
                            obj.get::<String>("msg").unwrap()
                        );
                    }
                }
            })?;
        }
    }
    Ok(())
}
