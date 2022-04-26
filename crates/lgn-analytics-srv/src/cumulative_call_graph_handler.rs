use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset};
use std::sync::Arc;

use lgn_analytics::{
    find_block_stream, find_process, find_process_thread_streams, find_stream_blocks_in_range,
    prelude::get_process_tick_length_ms, time::ConvertTicks,
};
use lgn_telemetry_proto::{
    analytics::{
        CallTreeNode, CumulativeCallGraphBlock, CumulativeCallGraphBlockDesc,
        CumulativeCallGraphComputedBlock, CumulativeCallGraphManifest,
    },
    telemetry::Process,
};
use lgn_tracing::span_fn;

use crate::{
    call_tree_store::CallTreeStore,
    cumulative_call_graph::tree_overlaps,
    cumulative_call_graph_node::{CallGraphNode, CallNodeHashMap},
    scope::{compute_scope_hash, ScopeHashMap},
};

pub struct CumulativeCallGraphHandler {
    pool: sqlx::any::AnyPool,
    call_tree_store: Arc<CallTreeStore>,
}

impl CumulativeCallGraphHandler {
    pub fn new(pool: sqlx::any::AnyPool, call_tree_store: Arc<CallTreeStore>) -> Self {
        Self {
            pool,
            call_tree_store,
        }
    }

    #[span_fn]
    #[allow(clippy::cast_precision_loss)]
    pub(crate) async fn get_process_call_graph_manifest(
        &self,
        process_id: String,
        begin_ms: f64,
        end_ms: f64,
    ) -> Result<CumulativeCallGraphManifest> {
        let mut connection = self.pool.acquire().await?;

        // For now child processes are not queried and as a result don't participate in cumulative call graph computations.
        let process = find_process(&mut connection, &process_id).await?;
        let time_range = Self::get_process_time_range(&process, begin_ms, end_ms)?;
        let begin = time_range.0.to_rfc3339();
        let end = time_range.1.to_rfc3339();

        let mut block_desc: Vec<CumulativeCallGraphBlockDesc> = vec![];

        let inv_tsc_frequency = get_process_tick_length_ms(&process);

        let streams = find_process_thread_streams(&mut connection, &process_id).await?;
        for s in streams {
            let blocks =
                find_stream_blocks_in_range(&mut connection, &s.stream_id, &begin, &end).await?;
            let data: Vec<CumulativeCallGraphBlockDesc> = blocks
                .iter()
                .map(|b| CumulativeCallGraphBlockDesc {
                    full: ((b.begin_ticks - process.start_ticks) as f64 * inv_tsc_frequency)
                        >= begin_ms
                        && ((b.end_ticks - process.start_ticks) as f64 * inv_tsc_frequency)
                            <= end_ms,
                    id: b.block_id.clone(),
                })
                .collect();
            block_desc.extend_from_slice(&data);
        }
        Ok(CumulativeCallGraphManifest {
            blocks: block_desc,
            start_ticks: process.start_ticks,
            tsc_frequency: process.tsc_frequency,
        })
    }

    #[span_fn]
    pub(crate) async fn get_call_graph_block(
        &self,
        block_id: String,
        start_ticks: i64,
        tsc_frequency: u64,
    ) -> Result<CumulativeCallGraphBlock> {
        let mut connection = self.pool.acquire().await?;
        let stream = find_block_stream(&mut connection, &block_id).await?;
        let convert_ticks = ConvertTicks::from_meta_data(start_ticks, tsc_frequency);
        let call_tree = self
            .call_tree_store
            .get_call_tree(convert_ticks, &stream, &block_id)
            .await?;
        Ok(CumulativeCallGraphBlock {
            call_tree: Some(call_tree),
            stream_hash: compute_scope_hash(&stream.stream_id),
            stream_name: match stream.properties.get("thread-name") {
                Some(x) => x.to_string(),
                None => String::from("Unknown"),
            },
        })
    }

    #[span_fn]
    pub(crate) async fn get_call_graph_computed_block(
        &self,
        block_id: String,
        start_ticks: i64,
        tsc_frequency: u64,
        begin_ms: f64,
        end_ms: f64,
    ) -> Result<CumulativeCallGraphComputedBlock> {
        let mut connection = self.pool.acquire().await?;
        let stream = find_block_stream(&mut connection, &block_id).await?;
        let convert_ticks = ConvertTicks::from_meta_data(start_ticks, tsc_frequency);
        let tree = self
            .call_tree_store
            .get_call_tree(convert_ticks, &stream, &block_id)
            .await?;

        let mut scopes = ScopeHashMap::new();
        let mut result = CallNodeHashMap::new();
        let mut full = true;
        if let Some(root) = tree.root {
            scopes.extend(tree.scopes);
            Self::build(&root, begin_ms, end_ms, &mut result, &mut full, None);
        }

        Ok(CumulativeCallGraphComputedBlock {
            full,
            scopes,
            nodes: result
                .iter()
                .map(|(_, node)| node.to_proto_node())
                .collect(),
            stream_hash: compute_scope_hash(&stream.stream_id),
            stream_name: match stream.properties.get("thread-name") {
                Some(x) => x.to_string(),
                None => String::from("Unknown"),
            },
        })
    }

    #[span_fn]
    fn build(
        tree: &CallTreeNode,
        begin_ms: f64,
        end_ms: f64,
        result: &mut CallNodeHashMap,
        full_flag: &mut bool,
        parent: Option<&CallTreeNode>,
    ) {
        if !tree_overlaps(tree, begin_ms, end_ms) {
            *full_flag = false;
            return;
        }

        let node = result
            .entry(tree.hash)
            .or_insert_with(|| CallGraphNode::new(tree.hash, begin_ms, end_ms));
        node.add_call(tree, parent);

        for child in &tree.children {
            Self::build(child, begin_ms, end_ms, result, full_flag, Some(tree));
        }
    }

    #[span_fn]
    fn get_process_time_range(
        process: &Process,
        begin_ms: f64,
        end_ms: f64,
    ) -> Result<(DateTime<FixedOffset>, DateTime<FixedOffset>)> {
        let start_time = chrono::DateTime::parse_from_rfc3339(&process.start_time)
            .with_context(|| String::from("parsing process start time"))?;
        let begin_offset_ns = begin_ms * 1_000_000.0;
        let begin_time = start_time + chrono::Duration::nanoseconds(begin_offset_ns as i64);
        let end_offset_ns = end_ms * 1_000_000.0;
        let end_time = start_time + chrono::Duration::nanoseconds(end_offset_ns as i64);
        Ok((begin_time, end_time))
    }
}
