use analytics::*;
use anyhow::*;
use sqlx::Row;
use std::path::{Path, PathBuf};
use test_utils::*;

static DUMP_EXE_VAR: &str = env!("CARGO_BIN_EXE_telemetry-dump");

fn test_dir(test_name: &str) -> PathBuf {
    let parent = Path::new(DUMP_EXE_VAR)
        .parent()
        .unwrap()
        .join("telemetry-dump-test-scratch");
    create_test_dir(&parent, test_name)
}

fn setup_data_dir(test_name: &str) -> PathBuf {
    let src_dir = std::env::current_dir().unwrap().join("tests/data");
    let test_output = test_dir(test_name);
    fs_extra::dir::copy(&src_dir, &test_output, &fs_extra::dir::CopyOptions::new()).unwrap();
    test_output.join("data")
}

fn dump_cli_sys(args: &[&str]) {
    syscall(DUMP_EXE_VAR, Path::new("."), args, true);
}

async fn find_process_with_log_data(connection: &mut sqlx::AnyConnection) -> Result<String> {
    let row = sqlx::query(
        "SELECT streams.process_id as process_id
         FROM streams, blocks
         WHERE streams.stream_id = blocks.stream_id
         AND tags LIKE '%log%';",
    )
    .fetch_one(connection)
    .await
    .with_context(|| "find_process_with_log_data")?;
    Ok(row.get("process_id"))
}

async fn find_block_with_thread_data(
    connection: &mut sqlx::AnyConnection,
) -> Result<telemetry::EncodedBlock> {
    let row = sqlx::query(
        "SELECT blocks.block_id as block_id, blocks.stream_id, begin_time, begin_ticks, end_time, end_ticks
         FROM streams, blocks
         WHERE streams.stream_id = blocks.stream_id
         AND tags LIKE '%cpu%';",
    )
    .fetch_one(connection)
    .await
        .with_context(|| "find_block_with_thread_data")?;

    let begin_ticks: i64 = row.get("begin_ticks");
    let end_ticks: i64 = row.get("end_ticks");
    Ok(telemetry::EncodedBlock {
        block_id: row.get("block_id"),
        stream_id: row.get("stream_id"),
        begin_time: row.get("begin_time"),
        begin_ticks: begin_ticks as u64,
        end_time: row.get("end_time"),
        end_ticks: end_ticks as u64,
        payload: None,
    })
}

#[test]
fn test_list_processes() {
    let data_path = setup_data_dir("list-processes");
    dump_cli_sys(&[data_path.to_str().unwrap(), "recent-processes"])
}

#[tokio::main]
#[test]
async fn test_print_log() -> Result<()> {
    let data_path = setup_data_dir("print-log");
    let pool = alloc_sql_pool(&data_path).await.unwrap();
    let mut connection = pool.acquire().await.unwrap();
    let process_id = find_process_with_log_data(&mut connection).await?;
    dump_cli_sys(&[data_path.to_str().unwrap(), "process-log", &process_id]);
    Ok(())
}

#[tokio::main]
#[test]
async fn test_thread_events() -> Result<()> {
    let data_path = setup_data_dir("thread-events");
    let pool = alloc_sql_pool(&data_path).await.unwrap();
    let mut connection = pool.acquire().await.unwrap();
    let block = find_block_with_thread_data(&mut connection).await?;
    let stream = find_stream(&mut connection, &block.stream_id).await?;
    let payload = fetch_block_payload(&mut connection, &data_path, &block.block_id).await?;
    parse_block(&stream, &payload, |val| {
        dbg!(val);
    })?;
    Ok(())
}
