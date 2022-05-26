use anyhow::{Context, Result};
use lgn_analytics::prelude::*;
use lgn_analytics::time::ConvertTicks;
use lgn_blob_storage::BlobStorage;
use lgn_tracing::info;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::channel;
use std::sync::Arc;

use deltalake::{
    action::Protocol, DeltaTable, DeltaTableMetaData, Schema, SchemaDataType, SchemaField,
};

use crate::call_tree::CallTreeBuilder;
use crate::lakehouse::span_table_partition::{
    make_rows_from_tree, SpanRowGroup, SpanTablePartitionLocalWriter,
};
use crate::thread_block_processor::parse_thread_block;

fn get_delta_schema() -> Schema {
    Schema::new(vec![
        SchemaField::new(
            "hash".to_string(),
            SchemaDataType::primitive("integer".to_string()),
            false,
            HashMap::new(),
        ),
        SchemaField::new(
            "depth".to_string(),
            SchemaDataType::primitive("integer".to_string()),
            false,
            HashMap::new(),
        ),
        SchemaField::new(
            "begin_ms".to_string(),
            SchemaDataType::primitive("double".to_string()),
            false,
            HashMap::new(),
        ),
        SchemaField::new(
            "end_ms".to_string(),
            SchemaDataType::primitive("double".to_string()),
            false,
            HashMap::new(),
        ),
        SchemaField::new(
            "id".to_string(),
            SchemaDataType::primitive("integer".to_string()),
            false,
            HashMap::new(),
        ),
        SchemaField::new(
            "parent".to_string(),
            SchemaDataType::primitive("integer".to_string()),
            false,
            HashMap::new(),
        ),
    ])
}

async fn make_delta_table(table_uri: &str) -> Result<DeltaTable> {
    info!("creating table {}", table_uri);
    let storage = deltalake::storage::get_backend_for_uri(table_uri)?;
    let mut table = deltalake::DeltaTable::new(
        table_uri,
        storage,
        deltalake::DeltaTableConfig {
            require_tombstones: false,
            require_files: false,
        },
    )?;
    let table_schema = get_delta_schema();
    let mut commit_info = serde_json::Map::<String, serde_json::Value>::new();
    commit_info.insert(
        "operation".to_string(),
        serde_json::Value::String("CREATE TABLE".to_string()),
    );
    let protocol = Protocol {
        min_reader_version: 1,
        min_writer_version: 1,
    };
    let metadata = DeltaTableMetaData::new(None, None, None, table_schema, vec![], HashMap::new());
    table
        .create(metadata, protocol, Some(commit_info), None)
        .await?;
    Ok(table)
}

#[allow(clippy::cast_possible_wrap)]
pub async fn make_span_partitions(
    pool: sqlx::any::AnyPool,
    blob_storage: Arc<dyn BlobStorage>,
    process_id: &str,
    convert_ticks: &ConvertTicks,
    spans_table_path: std::path::PathBuf,
) -> Result<()> {
    let mut handles = vec![];

    let (sender, receiver) = channel();

    let next_id = Arc::new(AtomicU64::new(1));
    let mut connection = pool.acquire().await?;
    let streams = find_process_thread_streams(&mut connection, process_id).await?;
    for stream in streams {
        let blocks = find_stream_blocks(&mut connection, &stream.stream_id).await?;
        for block in blocks {
            let convert_ticks = convert_ticks.clone();
            let mut connection = pool.acquire().await?;
            let blob_storage = blob_storage.clone();
            let stream = stream.clone();
            let next_id = next_id.clone();
            let spans_table_path = spans_table_path.clone();
            let sender = sender.clone();
            handles.push(tokio::spawn(async move {
                info!("processing block {}", &block.block_id);
                let mut builder =
                    CallTreeBuilder::new(block.begin_ticks, block.end_ticks, convert_ticks);
                parse_thread_block(
                    &mut connection,
                    blob_storage,
                    &stream,
                    block.block_id.clone(),
                    &mut builder,
                )
                .await?;
                let processed_block = builder.finish();
                if let Some(root) = processed_block.call_tree_root {
                    let mut rows = SpanRowGroup::new();
                    make_rows_from_tree(&root, &*next_id, &mut rows);
                    let filename = format!("spans_block_id={}.parquet", &block.block_id);
                    let parquet_full_path = spans_table_path.join(&filename);
                    let mut writer = SpanTablePartitionLocalWriter::create(&parquet_full_path)?;
                    writer.append(&rows)?;
                    writer.close()?;
                    let attr = std::fs::metadata(&parquet_full_path)?; //that's not cool, we should already know how big the file is
                    sender.send(deltalake::action::Action::add(deltalake::action::Add {
                        path: filename,
                        size: attr.len() as i64,
                        partition_values: HashMap::new(),
                        partition_values_parsed: None,
                        modification_time: 0,
                        data_change: false,
                        stats: None,
                        stats_parsed: None,
                        tags: None,
                    }))?;
                }
                Ok(()) as Result<(), anyhow::Error>
            }));
        }
    }
    drop(sender);
    for h in handles {
        h.await??;
    }

    let storage_uri = format!("{}", spans_table_path.display());
    let mut table = make_delta_table(&storage_uri).await?;
    let actions: Vec<deltalake::action::Action> = receiver.iter().collect();
    let mut transaction = table.create_transaction(None);
    transaction.add_actions(actions);
    transaction
        .commit(None, None)
        .await
        .with_context(|| "committing transaction")?;
    Ok(())
}
