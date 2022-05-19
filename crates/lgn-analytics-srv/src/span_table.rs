use anyhow::{Context, Result};
use lgn_analytics::prelude::*;
use lgn_analytics::time::ConvertTicks;
use lgn_blob_storage::BlobStorage;
use lgn_telemetry_proto::analytics::CallTreeNode;
use lgn_tracing::prelude::*;
use parquet::column::writer::ColumnWriter;
use parquet::file::properties::WriterProperties;
use parquet::file::writer::FileWriter;
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::parser::parse_message_type;
use std::path::Path;
use std::sync::Arc;

use crate::call_tree::CallTreeBuilder;
use crate::column::Column;
use crate::thread_block_processor::parse_thread_block;

#[derive(Debug)]
pub struct SpanTable {
    pub hashes: Column<i32>,
    pub depths: Column<i32>,
    pub begins: Column<f64>,
    pub ends: Column<f64>,
    pub ids: Column<u64>,
    pub parents: Column<u64>,
}

impl SpanTable {
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
        self.ids.append(row.id);
        self.parents.append(row.parent);
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
    next_id: &mut u64,
    process_row: &mut RowFun,
) where
    RowFun: FnMut(SpanRow),
{
    assert!(tree.hash != 0);
    let span_id = *next_id;
    *next_id += 1;
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

fn make_rows_from_tree(tree: &CallTreeNode, next_id: &mut u64, table: &mut SpanTable) {
    if tree.hash == 0 {
        for child in &tree.children {
            make_rows_from_tree_impl(child, 0, 0, next_id, &mut |row| table.append(&row));
        }
    } else {
        make_rows_from_tree_impl(tree, 0, 0, next_id, &mut |row| table.append(&row));
    }
}

pub async fn make_span_table(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    process_id: &str,
    convert_ticks: &ConvertTicks,
) -> Result<SpanTable> {
    let mut next_id = 1;
    let mut table = SpanTable::new();
    let streams = find_process_thread_streams(connection, process_id).await?;
    for stream in streams {
        let blocks = find_stream_blocks(connection, &stream.stream_id).await?;
        for block in blocks {
            let mut builder =
                CallTreeBuilder::new(block.begin_ticks, block.end_ticks, convert_ticks.clone());
            parse_thread_block(
                connection,
                blob_storage.clone(),
                &stream,
                block.block_id.clone(),
                &mut builder,
            )
            .await?;
            let processed = builder.finish();
            if let Some(root) = processed.call_tree_root {
                make_rows_from_tree(&root, &mut next_id, &mut table);
            }
        }
    }
    Ok(table)
}

#[span_fn]
pub fn write_parquet(file_path: &Path, spans: &SpanTable) -> Result<()> {
    let message_type = "
  message schema {
    REQUIRED INT32 hash;
    REQUIRED INT32 depth;
    REQUIRED DOUBLE begin_ms;
    REQUIRED DOUBLE end_ms;
  }
";
    let schema =
        Arc::new(parse_message_type(message_type).with_context(|| "parsing spans schema")?);
    let props = Arc::new(WriterProperties::builder().build());
    let file = std::fs::File::create(file_path)
        .with_context(|| format!("creating file {}", file_path.display()))?;
    let mut writer = SerializedFileWriter::new(file, schema, props)
        .with_context(|| "creating parquet writer")?;
    let mut row_group_writer = writer
        .next_row_group()
        .with_context(|| "creating row group writer")?;
    if let Some(mut col_writer) = row_group_writer
        .next_column()
        .with_context(|| "creating column writer")?
    {
        if let ColumnWriter::Int32ColumnWriter(writer_impl) = &mut col_writer {
            writer_impl
                .write_batch(&spans.hashes.values, None, None)
                .with_context(|| "writing hash batch")?;
        }
        row_group_writer
            .close_column(col_writer)
            .with_context(|| "closing column")?;
    }
    if let Some(mut col_writer) = row_group_writer
        .next_column()
        .with_context(|| "creating column writer")?
    {
        if let ColumnWriter::Int32ColumnWriter(writer_impl) = &mut col_writer {
            writer_impl
                .write_batch(&spans.depths.values, None, None)
                .with_context(|| "writing depth batch")?;
        }
        row_group_writer
            .close_column(col_writer)
            .with_context(|| "closing column")?;
    }
    if let Some(mut col_writer) = row_group_writer
        .next_column()
        .with_context(|| "creating column writer")?
    {
        if let ColumnWriter::DoubleColumnWriter(writer_impl) = &mut col_writer {
            writer_impl
                .write_batch(&spans.begins.values, None, None)
                .with_context(|| "writing begins batch")?;
        }
        row_group_writer
            .close_column(col_writer)
            .with_context(|| "closing column")?;
    }
    if let Some(mut col_writer) = row_group_writer
        .next_column()
        .with_context(|| "creating column writer")?
    {
        if let ColumnWriter::DoubleColumnWriter(writer_impl) = &mut col_writer {
            writer_impl
                .write_batch(&spans.ends.values, None, None)
                .with_context(|| "writing ends batch")?;
        }
        row_group_writer
            .close_column(col_writer)
            .with_context(|| "closing column")?;
    }
    writer
        .close_row_group(row_group_writer)
        .with_context(|| "closing row group")?;
    writer.close().with_context(|| "closing parquet writer")?;
    Ok(())
}
