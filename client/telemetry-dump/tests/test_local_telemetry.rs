use anyhow::*;
use legion_analytics::*;
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

async fn find_process_with_thread_data(connection: &mut sqlx::AnyConnection) -> Result<String> {
    let row = sqlx::query(
        "SELECT streams.process_id as process_id
         FROM streams, blocks
         WHERE streams.stream_id = blocks.stream_id
         AND tags LIKE '%cpu%';",
    )
    .fetch_one(connection)
    .await
    .with_context(|| "find_process_with_thread_data")?;
    Ok(row.get("process_id"))
}

async fn find_process_with_metrics_data(connection: &mut sqlx::AnyConnection) -> Result<String> {
    let row = sqlx::query(
        "SELECT streams.process_id as process_id
         FROM streams, blocks
         WHERE streams.stream_id = blocks.stream_id
         AND tags LIKE '%metrics%';",
    )
    .fetch_one(connection)
    .await
    .with_context(|| "find_process_with_metrics_data")?;
    Ok(row.get("process_id"))
}

#[test]
fn test_list_processes() {
    let data_path = setup_data_dir("list-processes");
    dump_cli_sys(&[data_path.to_str().unwrap(), "recent-processes"])
}

#[test]
fn test_find_processes() {
    let data_path = setup_data_dir("find-processes");
    dump_cli_sys(&[data_path.to_str().unwrap(), "find-processes", "exe"])
}

#[test]
fn test_logs_by_process() {
    let data_path = setup_data_dir("logs_by_process");
    dump_cli_sys(&[data_path.to_str().unwrap(), "logs-by-process"])
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
    let process_id = find_process_with_thread_data(&mut connection).await?;
    dump_cli_sys(&[
        data_path.to_str().unwrap(),
        "process-thread-events",
        &process_id,
    ]);
    Ok(())
}

#[tokio::main]
#[test]
async fn test_metrics_events() -> Result<()> {
    let data_path = setup_data_dir("metrics");
    let pool = alloc_sql_pool(&data_path).await.unwrap();
    let mut connection = pool.acquire().await.unwrap();
    let process_id = find_process_with_metrics_data(&mut connection).await?;
    dump_cli_sys(&[data_path.to_str().unwrap(), "process-metrics", &process_id]);
    Ok(())
}
