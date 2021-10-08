use anyhow::Result;
use legion_analytics::{
    fetch_block_payload, fetch_recent_processes, find_process_log_streams, find_stream_blocks,
    parse_block,
};
use std::path::Path;
use transit::Value;

pub async fn print_process_log(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    process_id: &str,
) -> Result<()> {
    for stream in find_process_log_streams(connection, process_id).await? {
        for b in find_stream_blocks(connection, &stream.stream_id).await? {
            let payload = fetch_block_payload(connection, data_path, &b.block_id).await?;
            parse_block(&stream, &payload, |val| {
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
            })?;
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
        println!();
    }
    Ok(())
}
