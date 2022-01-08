use std::collections::HashMap;
use std::{cmp::min, path::Path};

use anyhow::{Context, Result};
use lgn_analytics::prelude::*;
use lgn_telemetry::prelude::*;
use lgn_telemetry_proto::analytics::CallTreeNode;
use lgn_telemetry_proto::analytics::{
    CallGraphEdge, CumulativeCallGraphNode, CumulativeCallGraphReply, NodeStats,
};

use crate::call_tree::{compute_block_call_tree, ScopeHashMap}; //todo: move to analytics lib

struct NodeStatsAcc {
    durations_ms: Vec<f64>,
    parents: HashMap<u32, f64>,
    children: HashMap<u32, f64>,
}

impl NodeStatsAcc {
    #[trace_function]
    pub fn new() -> Self {
        Self {
            durations_ms: Vec::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
        }
    }
}

type StatsHashMap = std::collections::HashMap<u32, NodeStatsAcc>;

#[trace_function]
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

#[trace_function]
fn tree_overlaps(tree: &CallTreeNode, filter_begin_ms: f64, filter_end_ms: f64) -> bool {
    tree.end_ms >= filter_begin_ms && tree.begin_ms <= filter_end_ms
}

#[trace_function]
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

async fn record_process_call_graph(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
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
            //compute_block_call_tree fetches the block metadata again
            let tree =
                compute_block_call_tree(connection, data_path, process, &s, &b.block_id).await?;
            if let Some(root) = tree.root {
                scopes.extend(tree.scopes);
                record_tree_stats(&root, begin_ms, end_ms, stats, None);
            }
        }
    }
    Ok(())
}

#[allow(clippy::cast_precision_loss)]
pub(crate) async fn compute_cumulative_call_graph(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    process: &lgn_telemetry_sink::ProcessInfo,
    begin_ms: f64,
    end_ms: f64,
) -> Result<CumulativeCallGraphReply> {
    //this is a serial implementation, could be transformed in map/reduce
    let mut scopes = ScopeHashMap::new();
    let mut stats = StatsHashMap::new();
    record_process_call_graph(
        connection,
        data_path,
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
            data_path,
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
        let mut median = 0.0;
        if !node_stats.durations_ms.is_empty() {
            let index_median = min(
                node_stats.durations_ms.len() / 2,
                node_stats.durations_ms.len() - 1,
            );
            min_time = node_stats.durations_ms[0];
            max_time = node_stats.durations_ms[node_stats.durations_ms.len() - 1];
            median = node_stats.durations_ms[index_median];
        }
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
