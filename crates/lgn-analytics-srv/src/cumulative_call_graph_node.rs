use lgn_telemetry_proto::analytics::{
    CallTreeNode, CumulativeCallGraphEdge, CumulativeComputedCallGraphNode,
};
use lgn_tracing::span_fn;

use crate::cumulative_call_graph::tree_overlaps;

pub type CallNodeHashMap = std::collections::HashMap<u32, CallGraphNode>;

pub struct CallGraphNode {
    hash: u32,
    begin_ms: f64,
    end_ms: f64,
    sum: f64,
    sum_sqr: f64,
    min: f64,
    max: f64,
    count: u64,
    child_sum: f64,
    parents: CallNodeHashMap,
    children: CallNodeHashMap,
}

impl CallGraphNode {
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
            child_sum: 0.0,
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
        parent_node.process(parent);
        parent_node.child_sum += time_ms;
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
    pub fn to_proto_node(&self) -> CumulativeComputedCallGraphNode {
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

    #[span_fn]
    fn to_proto_edge(&self) -> CumulativeCallGraphEdge {
        CumulativeCallGraphEdge {
            hash: self.hash,
            count: self.count,
            max: self.max,
            min: self.min,
            sum: self.sum,
            sum_sqr: self.sum_sqr,
            child_sum: self.child_sum,
        }
    }
}
