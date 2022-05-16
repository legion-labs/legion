use std::sync::Arc;

use crate::{
    call_tree::CallTreeBuilder, jit_lakehouse::JitLakehouse,
    thread_block_processor::parse_thread_block,
};
use anyhow::Result;
use async_trait::async_trait;
use lgn_analytics::{prelude::*, time::ConvertTicks};
use lgn_blob_storage::BlobStorage;
use lgn_telemetry_proto::analytics::CallTreeNode;
use lgn_tracing::prelude::*;

pub struct LocalJitLakehouse {
    pool: sqlx::any::AnyPool,
    blob_storage: Arc<dyn BlobStorage>,
}

impl LocalJitLakehouse {
    pub fn new(pool: sqlx::any::AnyPool, blob_storage: Arc<dyn BlobStorage>) -> Self {
        Self { pool, blob_storage }
    }
}

#[derive(Debug)]
struct Column<T> {
    values: Vec<T>,
}

impl<T> Column<T> {
    pub fn new() -> Self {
        Self { values: vec![] }
    }

    pub fn append(&mut self, v: T) {
        self.values.push(v);
    }
}

#[derive(Debug)]
struct SpanTable {
    hashes: Column<u32>,
    depths: Column<u32>,
    begins: Column<f64>,
    ends: Column<f64>,
    ids: Column<u64>,
    parents: Column<u64>,
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

    pub fn append(&mut self, row: &SpanRow) {
        self.hashes.append(row.hash);
        self.depths.append(row.depth);
        self.begins.append(row.begin_ms);
        self.ends.append(row.end_ms);
        self.ids.append(row.id);
        self.parents.append(row.parent);
    }
}

#[derive(Debug)]
struct SpanRow {
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

#[async_trait]
impl JitLakehouse for LocalJitLakehouse {
    async fn build_timeline_tables(&self, process_id: &str) -> Result<()> {
        let mut connection = self.pool.acquire().await?;
        let process = find_process(&mut connection, process_id).await?;
        let convert_ticks = ConvertTicks::new(&process);
        warn!("build_timeline_tables {:?}", process);
        let mut next_id = 1;
        let mut table = SpanTable::new();
        let streams = find_process_thread_streams(&mut connection, process_id).await?;
        for stream in streams {
            let blocks = find_stream_blocks(&mut connection, &stream.stream_id).await?;
            for block in blocks {
                let mut builder =
                    CallTreeBuilder::new(block.begin_ticks, block.end_ticks, convert_ticks.clone());
                parse_thread_block(
                    &mut connection,
                    self.blob_storage.clone(),
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
        warn!("table: {:?}", table);
        Ok(())
    }
}
