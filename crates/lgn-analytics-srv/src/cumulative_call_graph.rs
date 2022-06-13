use lgn_telemetry_proto::analytics::CallTreeNode;
use lgn_tracing::span_fn;

#[span_fn]
pub fn tree_overlaps(tree: &CallTreeNode, filter_begin_ms: f64, filter_end_ms: f64) -> bool {
    tree.end_ms >= filter_begin_ms && tree.begin_ms <= filter_end_ms
}
