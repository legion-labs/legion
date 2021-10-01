use analytics::*;
use anyhow::*;
use prost::Message;
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

#[derive(Debug)]
enum Value {
    String(String),
}

fn parse_dependencies<F>(
    udts: &telemetry::telemetry_ingestion_proto::ContainerMetadata,
    buffer: &[u8],
    fun: F,
) where
    F: Fn(usize, Value),
{
    let mut offset = 0;
    while offset < buffer.len() {
        let type_index = buffer[offset] as usize;
        offset += 1;
        let udt = &udts.types[type_index];
        let object_size = match udt.size {
            0 => {
                //dynamic size
                unsafe {
                    let size_ptr = buffer.as_ptr().add(offset);
                    let obj_size = transit::read_pod::<u32>(size_ptr);
                    offset += std::mem::size_of::<u32>();
                    obj_size
                }
            }
            static_size => static_size,
        } as usize;
        dbg!(&object_size);
        match udt.name.as_str() {
            "StaticString" => unsafe {
                let id_ptr = buffer.as_ptr().add(offset);
                let string_id = transit::read_pod::<usize>(id_ptr);
                let nb_utf8_bytes = object_size - std::mem::size_of::<usize>();
                let utf8_ptr = buffer.as_ptr().add(offset + std::mem::size_of::<usize>());
                let slice = std::ptr::slice_from_raw_parts(utf8_ptr, nb_utf8_bytes);
                let string = String::from(std::str::from_utf8(&*slice).unwrap());
                fun(string_id, Value::String(string));
            },
            unknown_type => {
                println!("unknown type {}", unknown_type);
            }
        }
        offset += object_size;
    }
}

#[test]
fn test_list_processes() {
    let data_path = setup_data_dir("list-processes");
    dump_cli_sys(&[data_path.to_str().unwrap(), "recent-processes"])
}

async fn print_process_log(
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
            // dbg!(b);
            let payload = telemetry::telemetry_ingestion_proto::BlockPayload::decode(&*buffer)
                .with_context(|| format!("reading payload file {}", payload_path.display()))?;
            parse_dependencies(
                stream.dependencies_metadata.as_ref().unwrap(),
                &payload.dependencies,
                |id, value| {
                    dbg!(id);
                    dbg!(value);
                },
            );
        }
    }
    Ok(())
}

#[tokio::main]
#[test]
async fn test_print_log() -> Result<()> {
    let data_path = setup_data_dir("print-log");
    let pool = alloc_sql_pool(&data_path).await.unwrap();
    let mut connection = pool.acquire().await.unwrap();
    let process_id = find_process_with_log_data(&mut connection).await?;
    print_process_log(&mut connection, &data_path, &process_id).await?;
    Ok(())
}
