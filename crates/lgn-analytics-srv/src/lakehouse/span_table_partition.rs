use anyhow::{Context, Result};
use lgn_telemetry_proto::analytics::CallTreeNode;
use parquet::file::properties::WriterProperties;
use parquet::file::writer::FileWriter;
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::parser::parse_message_type;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use super::column::Column;

pub struct SpanTablePartitionLocalWriter {
    file_writer: SerializedFileWriter<std::fs::File>,
}

impl SpanTablePartitionLocalWriter {
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

pub fn make_rows_from_tree(tree: &CallTreeNode, next_id: &AtomicU64, table: &mut SpanRowGroup) {
    if tree.hash == 0 {
        for child in &tree.children {
            make_rows_from_tree_impl(child, 0, 0, next_id, &mut |row| table.append(&row));
        }
    } else {
        make_rows_from_tree_impl(tree, 0, 0, next_id, &mut |row| table.append(&row));
    }
}
