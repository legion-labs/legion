use analytics::*;
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

#[tokio::main]
#[test]
async fn test_list_processes() {
    let src_dir = std::env::current_dir().unwrap().join("tests/data");
    let test_output = test_dir("list-processes");
    fs_extra::dir::copy(&src_dir, &test_output, &fs_extra::dir::CopyOptions::new()).unwrap();

    let data_path = test_output.join("data");
    let pool = alloc_sql_pool(&data_path).await.unwrap();
    let mut connection = pool.acquire().await.unwrap();
    for p in fetch_recent_processes(&mut connection).await.unwrap() {
        println!("{} {}", p.start_time, p.exe);
    }
}
