use anyhow::{Context, Result};
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

pub async fn alloc_sql_pool(data_folder: &Path) -> Result<sqlx::AnyPool> {
    let db_uri = format!("sqlite://{}/telemetry.db3", data_folder.display());
    let pool = sqlx::any::AnyPoolOptions::new()
        .connect(&db_uri)
        .await
        .with_context(|| String::from("Connecting to telemetry database"))?;
    Ok(pool)
}

#[tokio::main]
#[test]
async fn test_list_processes() {
    let src_dir = std::env::current_dir().unwrap().join("tests/data");
    let test_output = test_dir("list-processes");
    fs_extra::dir::copy(&src_dir, &test_output, &fs_extra::dir::CopyOptions::new()).unwrap();

    let data_path = test_output.join("data");
    let pool = alloc_sql_pool(&data_path).await.unwrap();
    let mut connection = pool.acquire().await.unwrap();
    let rows = sqlx::query(
        "SELECT process_id, exe, start_time
         FROM processes",
    )
    .fetch_all(&mut connection)
    .await
    .unwrap();
    for r in rows {
        let process_id: String = r.get("process_id");
        let exe: String = r.get("exe");
        let start_time: String = r.get("start_time");
        println!("{} {} {}", process_id, exe, start_time);
    }
}
