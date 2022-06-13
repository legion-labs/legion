use anyhow::Result;
use lgn_telemetry_proto::analytics::CallTreeNode;
use lgn_telemetry_proto::analytics::Span;
use lgn_telemetry_proto::analytics::SpanBlockLod;
use lgn_telemetry_proto::analytics::SpanTrack;
use parquet::file::reader::ChunkReader;
use parquet::file::reader::FileReader;
use parquet::file::serialized_reader::SerializedFileReader;
use parquet::record::RowAccessor;
use std::path::Path;

use super::column::Column;
use super::column::TableColumn;
use super::parquet_buffer::write_to_file;
use super::parquet_buffer::ParquetBufferWriter;

pub fn make_spans_table_writer() -> Result<ParquetBufferWriter> {
    let schema = "message schema {
    REQUIRED INT32 hash;
    REQUIRED INT32 depth;
    REQUIRED DOUBLE begin_ms;
    REQUIRED DOUBLE end_ms;
    REQUIRED INT64 id;
    REQUIRED INT64 parent;
  }
";
    ParquetBufferWriter::create(schema)
}

pub async fn write_spans_parquet(rows: &SpanRowGroup, parquet_full_path: &Path) -> Result<()> {
    let mut writer = make_spans_table_writer()?;
    writer.write_row_group(&rows.get_columns())?;
    write_to_file(writer, parquet_full_path).await?;
    Ok(())
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

    pub fn get_columns(&self) -> Vec<&dyn TableColumn> {
        vec![
            &self.hashes,
            &self.depths,
            &self.begins,
            &self.ends,
            &self.ids,
            &self.parents,
        ]
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

pub fn make_rows_from_tree(tree: &CallTreeNode, next_id: &mut i64, table: &mut SpanRowGroup) {
    if tree.hash == 0 {
        for child in &tree.children {
            make_rows_from_tree_impl(child, 0, 0, next_id, &mut |row| table.append(&row));
        }
    } else {
        make_rows_from_tree_impl(tree, 0, 0, next_id, &mut |row| table.append(&row));
    }
}

pub fn read_spans<R: 'static + ChunkReader>(
    file_reader: &SerializedFileReader<R>,
) -> Result<SpanBlockLod> {
    let mut lod = SpanBlockLod {
        lod_id: 0,
        tracks: vec![],
    };
    for row in file_reader.get_row_iter(None)? {
        let hash = row.get_int(0)?;
        let depth = row.get_int(1)?;
        let begin = row.get_double(2)?;
        let end = row.get_double(3)?;
        if lod.tracks.len() <= depth as usize {
            lod.tracks.push(SpanTrack { spans: vec![] });
        }
        let span = Span {
            scope_hash: hash as u32,
            begin_ms: begin,
            end_ms: end,
            alpha: 255,
        };
        lod.tracks[depth as usize].spans.push(span);
    }
    Ok(lod)
}
