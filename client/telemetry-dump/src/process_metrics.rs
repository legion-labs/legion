use anyhow::*;
use legion_analytics::*;
use std::path::Path;
use transit::*;

pub async fn print_process_metrics(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    process_id: &str,
) -> Result<()> {
    for stream in find_process_metrics_streams(connection, process_id).await? {
        println!("stream {}", stream.stream_id);
        for block in find_stream_blocks(connection, &stream.stream_id).await? {
            println!("block {}", block.block_id);
            let payload = fetch_block_payload(connection, data_path, &block.block_id).await?;
            parse_block(&stream, &payload, |val| {
                if let Value::Object(obj) = val {
                    let metric = obj.get::<Object>("metric").unwrap();
                    let name = metric.get::<String>("name").unwrap();
                    let unit = metric.get::<String>("unit").unwrap();
                    if let Ok(int_value) = obj.get::<u64>("value") {
                        println!("{} ({}) : {}", name, unit, int_value);
                    } else if let Ok(float_value) = obj.get::<f64>("value") {
                        println!("{} ({}) : {}", name, unit, float_value);
                    }
                }
                true //continue
            })?;
            println!();
        }
        println!();
    }
    Ok(())
}
