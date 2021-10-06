use analytics::*;
use anyhow::*;
use prost::Message;
use std::io::Read;
use std::path::Path;
use transit::*;

pub fn lz4_decompress(compressed: &[u8]) -> Result<Vec<u8>> {
    let mut decompressed = Vec::new();
    let mut decoder = lz4::Decoder::new(compressed).with_context(|| "allocating lz4 decoder")?;
    let _size = decoder
        .read_to_end(&mut decompressed)
        .with_context(|| "reading lz4-compressed buffer")?;
    let (_reader, res) = decoder.finish();
    res?;
    Ok(decompressed)
}

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

            let dependencies = read_dependencies(
                &dep_udts,
                &lz4_decompress(&payload.dependencies)
                    .with_context(|| "decompressing dependencies payload")?,
            )?;
            let obj_udts = stream
                .objects_metadata
                .as_ref()
                .unwrap()
                .as_transit_udt_vec();
            parse_objects(
                &dependencies,
                &obj_udts,
                &lz4_decompress(&payload.objects)
                    .with_context(|| "decompressing objects payload")?,
                |val| {
                    if let Value::Object(obj) = val {
                        match obj.type_name.as_str() {
                            "LogMsgEvent" | "LogDynMsgEvent" => {
                                println!(
                                    "[{}] {}",
                                    obj.get::<u8>("level").unwrap(),
                                    obj.get::<String>("msg").unwrap()
                                );
                            }
                            _ => {}
                        }
                    }
                },
            )?;
        }
    }
    Ok(())
}

pub async fn print_logs_by_process(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
) -> Result<()> {
    for p in fetch_recent_processes(connection).await.unwrap() {
        println!("{} {} {}", p.start_time, p.process_id, p.exe);
        print_process_log(connection, data_path, &p.process_id).await?;
    }
    Ok(())
}
