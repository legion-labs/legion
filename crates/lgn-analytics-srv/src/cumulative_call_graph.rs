use std::cmp::min;
use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset};
use lgn_analytics::prelude::*;
use lgn_analytics::time::ConvertTicks;
use lgn_telemetry_proto::analytics::{
    CallGraphComputedEdge, CallGraphEdge, CallTreeNode, CumulativeCallGraphBlock,
    CumulativeCallGraphComputedBlock, CumulativeCallGraphManifest, CumulativeCallGraphNode,
    CumulativeCallGraphReply, CumulativeComputedCallGraphNode, NodeCumulativeStats, NodeStats,
};
use lgn_telemetry_proto::telemetry::Process;
use lgn_tracing::prelude::*;

use std::sync::Arc;

use crate::call_tree_store::CallTreeStore;
use crate::scope::{compute_scope_hash, ScopeHashMap};

struct NodeStatsAcc {
    durations_ms: Vec<f64>,
    parents: HashMap<u32, f64>,
    children: HashMap<u32, f64>,
}

impl NodeStatsAcc {
    #[span_fn]
    pub fn new() -> Self {
        Self {
            durations_ms: Vec::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
        }
    }
}

type StatsHashMap = std::collections::HashMap<u32, NodeStatsAcc>;

type CallNodeHashMap = std::collections::HashMap<u32, CallNode>;

pub struct CallNode {
    hash: u32,
    begin_ms: f64,
    end_ms: f64,
    sum: f64,
    sum_sqr: f64,
    min: f64,
    max: f64,
    count: u64,
    child_acc: f64,
    parents: CallNodeHashMap,
    children: CallNodeHashMap,
}

impl CallNode {
    #[span_fn]
    pub fn new(hash: u32, begin_ms: f64, end_ms: f64) -> Self {
        Self {
            hash,
            sum: 0.0,
            sum_sqr: 0.0,
            begin_ms,
            end_ms,
            min: f64::MAX,
            max: f64::MIN,
            count: 0,
            child_acc: 0.0,
            parents: CallNodeHashMap::new(),
            children: CallNodeHashMap::new(),
        }
    }

    #[span_fn]
    pub fn add_call(&mut self, node: &CallTreeNode, parent: Option<&CallTreeNode>) {
        let time_ms = self.process(node);
        if let Some(parent) = parent {
            self.add_parent_call(parent, time_ms);
        }
        for child in &node.children {
            if tree_overlaps(child, self.begin_ms, self.end_ms) {
                self.add_child_call(child);
            }
        }
    }

    #[span_fn]
    fn process(&mut self, node: &CallTreeNode) -> f64 {
        let time_ms = node.end_ms.min(self.end_ms) - node.begin_ms.max(self.begin_ms);
        self.sum += time_ms;
        self.sum_sqr += time_ms.powf(2.0);
        self.min = self.min.min(time_ms);
        self.max = self.max.max(time_ms);
        self.count += 1;
        time_ms
    }

    #[span_fn]
    fn add_parent_call(&mut self, parent: &CallTreeNode, time_ms: f64) {
        let parent_node = self
            .parents
            .entry(parent.hash)
            .or_insert_with(|| Self::new(parent.hash, self.begin_ms, self.end_ms));
        parent_node.add_call(parent, None);
        parent_node.child_acc += time_ms;
    }

    #[span_fn]
    fn add_child_call(&mut self, child: &CallTreeNode) {
        let child_node = self
            .children
            .entry(child.hash)
            .or_insert_with(|| Self::new(child.hash, self.begin_ms, self.end_ms));
        child_node.process(child);
    }

    #[span_fn]
    fn to_proto_edge(&self) -> CallGraphComputedEdge {
        CallGraphComputedEdge {
            hash: self.hash,
            stats: Some(NodeCumulativeStats {
                count: self.count,
                max: self.max,
                min: self.min,
                sum: self.sum,
                sum_sqr: self.sum_sqr,
            }),
        }
    }

    #[span_fn]
    fn to_proto_node(&self) -> CumulativeComputedCallGraphNode {
        CumulativeComputedCallGraphNode {
            node: Some(self.to_proto_edge()),
            callees: self
                .children
                .iter()
                .map(|(_, node)| node.to_proto_edge())
                .collect(),
            callers: self
                .parents
                .iter()
                .map(|(_, node)| node.to_proto_edge())
                .collect(),
        }
    }
}

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

        let mut block_ids = vec![];

        let streams = find_process_thread_streams(&mut connection, &process_id).await?;
        for s in streams {
            let blocks =
                find_stream_blocks_in_range(&mut connection, &s.stream_id, &begin, &end).await?;
            let data: Vec<String> = blocks.iter().map(|b| b.block_id.clone()).collect();
            block_ids.extend_from_slice(&data);
        }
        Ok(CumulativeCallGraphManifest {
            blocks: block_ids,
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
        if let Some(root) = tree.root {
            scopes.extend(tree.scopes);
            Self::build(&root, begin_ms, end_ms, &mut result, None);
        }

        Ok(CumulativeCallGraphComputedBlock {
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
        parent: Option<&CallTreeNode>,
    ) {
        if !tree_overlaps(tree, begin_ms, end_ms) {
            return;
        }

        let node = result
            .entry(tree.hash)
            .or_insert_with(|| CallNode::new(tree.hash, begin_ms, end_ms));
        node.add_call(tree, parent);

        for child in &tree.children {
            Self::build(child, begin_ms, end_ms, result, Some(tree));
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

#[span_fn]
fn make_edge_vector(edges_acc: &HashMap<u32, f64>) -> Vec<CallGraphEdge> {
    let mut edges: Vec<CallGraphEdge> = edges_acc
        .iter()
        .filter(|(hash, _weight)| **hash != 0)
        .map(|(hash, weight)| CallGraphEdge {
            hash: *hash,
            weight: *weight,
        })
        .collect();
    edges.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
    edges
}

#[span_fn]
fn tree_overlaps(tree: &CallTreeNode, filter_begin_ms: f64, filter_end_ms: f64) -> bool {
    tree.end_ms >= filter_begin_ms && tree.begin_ms <= filter_end_ms
}

#[span_fn]
fn record_tree_stats(
    tree: &CallTreeNode,
    filter_begin_ms: f64,
    filter_end_ms: f64,
    stats_map: &mut StatsHashMap,
    parent_hash: Option<u32>,
) {
    if !tree_overlaps(tree, filter_begin_ms, filter_end_ms) {
        return;
    }
    {
        let stats = stats_map.entry(tree.hash).or_insert_with(NodeStatsAcc::new);
        let duration = tree.end_ms.min(filter_end_ms) - tree.begin_ms.max(filter_begin_ms);
        stats.durations_ms.push(duration);
        if let Some(ph) = parent_hash {
            *stats.parents.entry(ph).or_insert(0.0) += duration;
        }
        for child in &tree.children {
            if tree_overlaps(child, filter_begin_ms, filter_end_ms) {
                let child_duration =
                    child.end_ms.min(filter_end_ms) - child.begin_ms.max(filter_begin_ms);
                *stats.children.entry(child.hash).or_insert(0.0) += child_duration;
            }
        }
    }
    for child in &tree.children {
        record_tree_stats(
            child,
            filter_begin_ms,
            filter_end_ms,
            stats_map,
            Some(tree.hash),
        );
    }
}

#[span_fn]
async fn record_process_call_graph(
    connection: &mut sqlx::AnyConnection,
    call_trees: &CallTreeStore,
    process: &lgn_telemetry_sink::ProcessInfo,
    begin_ms: f64,
    end_ms: f64,
    scopes: &mut ScopeHashMap,
    stats: &mut StatsHashMap,
) -> Result<()> {
    let start_time = chrono::DateTime::parse_from_rfc3339(&process.start_time)
        .with_context(|| String::from("parsing process start time"))?;
    let begin_offset_ns = begin_ms * 1_000_000.0;
    let begin_time = start_time + chrono::Duration::nanoseconds(begin_offset_ns as i64);

    let end_offset_ns = end_ms * 1_000_000.0;
    let end_time = start_time + chrono::Duration::nanoseconds(end_offset_ns as i64);
    let streams = find_process_thread_streams(connection, &process.process_id).await?;
    for s in streams {
        let blocks = find_stream_blocks_in_range(
            connection,
            &s.stream_id,
            &begin_time.to_rfc3339(),
            &end_time.to_rfc3339(),
        )
        .await?;

        for b in blocks {
            let tree = call_trees
                .get_call_tree(ConvertTicks::new(process), &s, &b.block_id)
                .await?;
            if let Some(root) = tree.root {
                scopes.extend(tree.scopes);
                record_tree_stats(&root, begin_ms, end_ms, stats, None);
            }
        }
    }
    Ok(())
}

#[allow(clippy::cast_precision_loss)]
#[span_fn]
pub(crate) async fn compute_cumulative_call_graph(
    connection: &mut sqlx::AnyConnection,
    call_trees: &CallTreeStore,
    process: &lgn_telemetry_sink::ProcessInfo,
    begin_ms: f64,
    end_ms: f64,
) -> Result<CumulativeCallGraphReply> {
    //this is a serial implementation, could be transformed in map/reduce
    let mut scopes = ScopeHashMap::new();
    let mut stats = StatsHashMap::new();
    record_process_call_graph(
        connection,
        call_trees,
        process,
        begin_ms,
        end_ms,
        &mut scopes,
        &mut stats,
    )
    .await?;

    let root_start_time = chrono::DateTime::parse_from_rfc3339(&process.start_time)
        .with_context(|| String::from("parsing process start time"))?;

    for child_process in fetch_child_processes(connection, &process.process_id).await? {
        let child_start_time = chrono::DateTime::parse_from_rfc3339(&child_process.start_time)
            .with_context(|| String::from("parsing process start time"))?;
        // how sensitive is this code to numerical instability?
        let time_offset = child_start_time - root_start_time;
        let time_offset_ms = time_offset.num_milliseconds() as f64;
        let time_offset_ns = (time_offset.num_nanoseconds().unwrap() % 1_000_000) as f64;
        let time_offset_total = time_offset_ms + (time_offset_ns / 1_000_000.0);
        record_process_call_graph(
            connection,
            call_trees,
            &child_process,
            begin_ms - time_offset_total,
            end_ms - time_offset_total,
            &mut scopes,
            &mut stats,
        )
        .await?;
    }

    let mut nodes = vec![];
    nodes.reserve(stats.len());
    for (hash, mut node_stats) in stats.drain() {
        let mut min_time = f64::MAX;
        let mut max_time = f64::MIN;
        let mut sum = 0.0;
        node_stats
            .durations_ms
            .sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if !node_stats.durations_ms.is_empty() {
            let index_median = min(
                node_stats.durations_ms.len() / 2,
                node_stats.durations_ms.len() - 1,
            );
            min_time = node_stats.durations_ms[0];
            max_time = node_stats.durations_ms[node_stats.durations_ms.len() - 1];
            node_stats.durations_ms[index_median]
        } else {
            0.0
        };
        for time_ms in &node_stats.durations_ms {
            sum += time_ms;
        }

        let callers = make_edge_vector(&node_stats.parents);
        let callees = make_edge_vector(&node_stats.children);

        nodes.push(CumulativeCallGraphNode {
            hash,
            stats: Some(NodeStats {
                sum,
                min: min_time,
                max: max_time,
                avg: sum / node_stats.durations_ms.len() as f64,
                median,
                count: node_stats.durations_ms.len() as u64,
            }),
            callers,
            callees,
        });
    }

    Ok(CumulativeCallGraphReply { scopes, nodes })
}
