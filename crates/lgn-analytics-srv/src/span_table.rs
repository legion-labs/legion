use anyhow::{Context, Result};
use lgn_analytics::prelude::*;
use lgn_analytics::time::ConvertTicks;
use lgn_blob_storage::BlobStorage;
use lgn_telemetry_proto::analytics::CallTreeNode;
use lgn_tracing::info;
use parquet::file::properties::WriterProperties;
use parquet::file::writer::FileWriter;
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::parser::parse_message_type;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::Arc;

use deltalake::{
    action::Protocol, DeltaTable, DeltaTableMetaData, Schema, SchemaDataType, SchemaField,
};

use crate::call_tree::CallTreeBuilder;
use crate::column::Column;
use crate::thread_block_processor::parse_thread_block;

pub struct SpanTableLocalWriter {
    file_writer: SerializedFileWriter<std::fs::File>,
}

impl SpanTableLocalWriter {
    pub fn create(file_path: &Path) -> Result<Self> {
        let message_type = "
  message schema {
    REQUIRED INT32 hash;
    REQUIRED INT32 depth;
    REQUIRED DOUBLE begin_ms;
    REQUIRED DOUBLE end_ms;
    REQUIRED INT64 id;
    REQUIRED INT64 parent;
  }
";
        let schema =
            Arc::new(parse_message_type(message_type).with_context(|| "parsing spans schema")?);
        let props = Arc::new(WriterProperties::builder().build());
        let file = std::fs::File::create(file_path)
            .with_context(|| format!("creating file {}", file_path.display()))?;
        let file_writer = SerializedFileWriter::new(file, schema, props)
            .with_context(|| "creating parquet writer")?;
        Ok(Self { file_writer })
    }

    pub fn close(mut self) -> Result<()> {
        self.file_writer.close()?;
        Ok(())
    }

    pub fn append(&mut self, spans: &SpanRowGroup) -> Result<()> {
        let mut row_group_writer = self
            .file_writer
            .next_row_group()
            .with_context(|| "creating row group writer")?;
        spans
            .hashes
            .write_batch(&mut *row_group_writer)
            .with_context(|| "writing hash column")?;
        spans
            .depths
            .write_batch(&mut *row_group_writer)
            .with_context(|| "writing depth column")?;
        spans
            .begins
            .write_batch(&mut *row_group_writer)
            .with_context(|| "writing begins column")?;
        spans
            .ends
            .write_batch(&mut *row_group_writer)
            .with_context(|| "writing begins column")?;
        spans
            .ids
            .write_batch(&mut *row_group_writer)
            .with_context(|| "writing ids column")?;
        spans
            .parents
            .write_batch(&mut *row_group_writer)
            .with_context(|| "writing parents column")?;
        self.file_writer
            .close_row_group(row_group_writer)
            .with_context(|| "closing row group")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SpanRowGroup {
    pub hashes: Column<i32>,
    pub depths: Column<i32>,
    pub begins: Column<f64>,
    pub ends: Column<f64>,
    pub ids: Column<i64>,
    pub parents: Column<i64>,
}

impl SpanRowGroup {
    pub fn new() -> Self {
        Self {
            hashes: Column::new(),
            depths: Column::new(),
            begins: Column::new(),
            ends: Column::new(),
            ids: Column::new(),
            parents: Column::new(),
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    pub fn append(&mut self, row: &SpanRow) {
        self.hashes.append(row.hash as i32);
        self.depths.append(row.depth as i32);
        self.begins.append(row.begin_ms);
        self.ends.append(row.end_ms);
        self.ids.append(row.id as i64);
        self.parents.append(row.parent as i64);
    }
}

#[derive(Debug)]
pub struct SpanRow {
    hash: u32,
    depth: u32,
    begin_ms: f64,
    end_ms: f64,
    id: u64,
    parent: u64,
}

fn make_rows_from_tree_impl<RowFun>(
    tree: &CallTreeNode,
    parent: u64,
    depth: u32,
    next_id: &AtomicU64,
    process_row: &mut RowFun,
) where
    RowFun: FnMut(SpanRow),
{
    assert!(tree.hash != 0);
    let span_id = next_id.fetch_add(1, Ordering::Relaxed);
    let span = SpanRow {
        hash: tree.hash,
        depth,
        begin_ms: tree.begin_ms,
        end_ms: tree.end_ms,
        id: span_id,
        parent,
    };
    process_row(span);
    for child in &tree.children {
        make_rows_from_tree_impl(child, span_id, depth + 1, next_id, process_row);
    }
}

fn make_rows_from_tree(tree: &CallTreeNode, next_id: &AtomicU64, table: &mut SpanRowGroup) {
    if tree.hash == 0 {
        for child in &tree.children {
            make_rows_from_tree_impl(child, 0, 0, next_id, &mut |row| table.append(&row));
        }
    } else {
        make_rows_from_tree_impl(tree, 0, 0, next_id, &mut |row| table.append(&row));
    }
}

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
                    let mut writer =
                        SpanTableLocalWriter::create(&spans_table_path.join(&filename))?;
                    writer.append(&rows)?;
                    writer.close()?;
                    sender.send(filename)?;
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
    let actions: Vec<deltalake::action::Action> = receiver
        .iter()
        .map(|f| {
            deltalake::action::Action::add(deltalake::action::Add {
                path: f,
                size: 0,
                partition_values: HashMap::new(),
                partition_values_parsed: None,
                modification_time: 0,
                data_change: false,
                stats: None,
                stats_parsed: None,
                tags: None,
            })
        })
        .collect();
    let mut transaction = table.create_transaction(None);
    transaction.add_actions(actions);
    transaction
        .commit(None, None)
        .await
        .with_context(|| "committing transaction")?;
    Ok(())
}
