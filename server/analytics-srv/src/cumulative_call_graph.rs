use crate::call_tree::{compute_block_call_tree, record_scope_in_map, CallTreeNode, ScopeHashMap}; //todo: move to analytics lib
use anyhow::{Context, Result};
use legion_analytics::prelude::*;
use legion_telemetry_proto::analytics::{
    CallGraphEdge, CumulativeCallGraphNode, CumulativeCallGraphReply, NodeStats,
};
use std::collections::HashMap;
use std::{cmp::min, path::Path};

struct NodeStatsAcc {
    durations_ms: Vec<f64>,
    parents: HashMap<u32, f64>,
    children: HashMap<u32, f64>,
}

impl NodeStatsAcc {
    pub fn new() -> Self {
        Self {
            durations_ms: Vec::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
        }
    }
}

type StatsHashMap = std::collections::HashMap<u32, NodeStatsAcc>;

fn record_tree_stats(
    tree: &CallTreeNode,
    scopes: &mut ScopeHashMap,
    stats_map: &mut StatsHashMap,
    parent_hash: Option<u32>,
) {
    record_scope_in_map(tree, scopes);
    {
        let stats = stats_map.entry(tree.hash).or_insert_with(NodeStatsAcc::new);
        let duration = tree.end_ms - tree.begin_ms;
        stats.durations_ms.push(duration);
        if let Some(ph) = parent_hash {
            *stats.parents.entry(ph).or_insert(0.0) += duration;
        }
        for child in &tree.scopes {
            let child_duration = child.end_ms - child.begin_ms;
            *stats.children.entry(child.hash).or_insert(0.0) += child_duration;
        }
    }
    for child in &tree.scopes {
        record_tree_stats(child, scopes, stats_map, Some(tree.hash));
    }
}

#[allow(clippy::cast_precision_loss)]
pub(crate) async fn compute_cumulative_call_graph(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    process: &legion_telemetry::ProcessInfo,
    begin_ms: f64,
    end_ms: f64,
) -> Result<CumulativeCallGraphReply> {
    //todo: include child processes
    //this is a serial implementation, could be transformed in map/reduce
    dbg!(&process.start_time);
    let start_time = chrono::DateTime::parse_from_rfc3339(&process.start_time)
        .with_context(|| String::from("parsing process start time"))?;
    dbg!(&start_time);
    let begin_offset_ns = begin_ms * 1_000_000.0;
    let begin_time = start_time + chrono::Duration::nanoseconds(begin_offset_ns as i64);
    dbg!(begin_time);

    let end_offset_ns = end_ms * 1_000_000.0;
    let end_time = start_time + chrono::Duration::nanoseconds(end_offset_ns as i64);
    dbg!(end_time);

    let streams = find_process_thread_streams(connection, &process.process_id).await?;
    let mut scopes = ScopeHashMap::new();
    let mut stats = StatsHashMap::new();
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
            //todo: filter individual call instances to count only those between begin_ms and end_ms
            let tree =
                compute_block_call_tree(connection, data_path, process, &s, &b.block_id).await?;
            record_tree_stats(&tree, &mut scopes, &mut stats, None);
        }
    }

    let mut scope_vec = vec![];
    scope_vec.reserve(scopes.len());
    for (_k, v) in scopes.drain() {
        scope_vec.push(v);
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

        let callers = node_stats
            .parents
            .iter()
            .map(|(hash, weight)| CallGraphEdge {
                hash: *hash,
                weight: *weight,
            })
            .collect();
        let callees = node_stats
            .children
            .iter()
            .map(|(hash, weight)| CallGraphEdge {
                hash: *hash,
                weight: *weight,
            })
            .collect();

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

    Ok(CumulativeCallGraphReply {
        scopes: scope_vec,
        nodes,
    })
}
