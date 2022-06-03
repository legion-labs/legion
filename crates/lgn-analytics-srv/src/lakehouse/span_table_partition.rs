use anyhow::{Context, Result};
use lgn_analytics::time::ConvertTicks;
use lgn_telemetry_proto::analytics::CallTreeNode;
use lgn_telemetry_proto::telemetry::BlockMetadata;
use lgn_telemetry_proto::telemetry::Stream;
use lgn_tracing::prelude::*;
use parquet::file::properties::WriterProperties;
use parquet::file::writer::FileWriter;
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::parser::parse_message_type;
use std::collections::HashMap;
use std::io::Cursor;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use crate::call_tree::CallTreeBuilder;
use crate::thread_block_processor::parse_thread_block_payload;

use super::column::Column;
use super::parquet_buffer::InMemStream;

pub struct SpanTablePartitionLocalWriter {
    buffer: Arc<Cursor<Vec<u8>>>,
    file_writer: SerializedFileWriter<InMemStream>,
    file_path: PathBuf,
}

impl SpanTablePartitionLocalWriter {
    #[span_fn]
    pub fn create(file_path: PathBuf) -> Result<Self> {
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
        let buffer = Arc::new(Cursor::new(Vec::new()));
        let file_writer =
            SerializedFileWriter::new(InMemStream::new(buffer.clone()), schema, props)
                .with_context(|| "creating parquet writer")?;
        Ok(Self {
            buffer,
            file_writer,
            file_path,
        })
    }

    #[span_fn]
    pub fn close(mut self) -> Result<()> {
        self.file_writer.close()?;
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&self.file_path)
            .with_context(|| format!("creating file {}", self.file_path.display()))?;
        file.write_all(self.buffer.get_ref())?;

        Ok(())
    }

    #[span_fn]
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
    id: i64,
    parent: i64,
}

fn make_rows_from_tree_impl<RowFun>(
    tree: &CallTreeNode,
    parent: i64,
    depth: u32,
    next_id: &mut i64,
    process_row: &mut RowFun,
) where
    RowFun: FnMut(SpanRow),
{
    assert!(tree.hash != 0);
    *next_id += 1;
    let span_id = *next_id;
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

pub fn make_rows_from_tree(tree: &CallTreeNode, next_id: &mut i64, table: &mut SpanRowGroup) {
    if tree.hash == 0 {
        for child in &tree.children {
            make_rows_from_tree_impl(child, 0, 0, next_id, &mut |row| table.append(&row));
        }
    } else {
        make_rows_from_tree_impl(tree, 0, 0, next_id, &mut |row| table.append(&row));
    }
}

#[allow(clippy::cast_possible_wrap)]
#[span_fn]
pub async fn write_local_partition(
    payload: &lgn_telemetry_proto::telemetry::BlockPayload,
    stream: &Stream,
    block: &BlockMetadata,
    convert_ticks: ConvertTicks,
    next_id: &mut i64,
    relative_file_name: String,
    parquet_full_path: &Path,
) -> Result<Option<deltalake::action::Action>> {
    //todo: do not allow overwriting - it could break id generation
    info!("processing block {}", &block.block_id);
    if let Some(parent) = parquet_full_path.parent() {
        tokio::fs::create_dir_all(&parent)
            .await
            .with_context(|| format!("creating directory for {}", parquet_full_path.display()))?;
    }
    let mut builder = CallTreeBuilder::new(block.begin_ticks, block.end_ticks, convert_ticks);
    parse_thread_block_payload(payload, stream, &mut builder)
        .with_context(|| "parsing thread block payload")?;
    let processed_block = builder.finish();
    if let Some(root) = processed_block.call_tree_root {
        let mut rows = SpanRowGroup::new();
        make_rows_from_tree(&root, next_id, &mut rows);
        let mut writer = SpanTablePartitionLocalWriter::create(parquet_full_path.to_path_buf())?;
        writer.append(&rows)?;
        writer.close()?;
        let attr = tokio::fs::metadata(&parquet_full_path).await?; //that's not cool, we should already know how big the file is
        Ok(Some(deltalake::action::Action::add(
            deltalake::action::Add {
                path: relative_file_name,
                size: attr.len() as i64,
                partition_values: HashMap::from([
                    ("block_id".to_owned(), Some(block.block_id.clone())),
                    ("thread_id".to_owned(), Some(stream.stream_id.clone())),
                ]),
                partition_values_parsed: None,
                modification_time: 0,
                data_change: false,
                stats: None,
                stats_parsed: None,
                tags: None,
            },
        )))
    } else {
        Ok(None)
    }
}
