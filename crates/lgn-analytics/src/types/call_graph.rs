use std::collections::HashMap;

use super::ScopeDesc;

#[derive(Clone, PartialEq)]
pub struct CumulativeCallGraphManifestRequest {
    pub process_id: String,
    pub begin_ms: f64,
    pub end_ms: f64,
}

#[derive(Clone, PartialEq)]
pub struct CumulativeCallGraphManifest {
    pub blocks: Vec<CumulativeCallGraphBlockDesc>,
    pub tsc_frequency: u64,
    pub start_ticks: i64,
}

#[derive(Clone, PartialEq)]
pub struct CumulativeCallGraphBlockDesc {
    pub id: String,
    pub full: bool,
}

#[derive(Clone, PartialEq)]
pub struct CumulativeCallGraphBlockRequest {
    pub block_id: String,
    pub begin_ms: f64,
    pub end_ms: f64,
    pub tsc_frequency: u64,
    pub start_ticks: i64,
}

#[derive(Clone, PartialEq)]
pub struct CumulativeCallGraphComputedBlock {
    pub scopes: HashMap<u32, ScopeDesc>,
    pub nodes: Vec<CumulativeComputedCallGraphNode>,
    pub stream_hash: u32,
    pub stream_name: String,
    pub full: bool,
}

#[derive(Clone, PartialEq)]
pub struct CumulativeComputedCallGraphNode {
    pub stats: ::core::option::Option<CumulativeStats>,
    pub callers: Vec<CumulativeStats>,
    pub callees: Vec<CumulativeStats>,
}

#[derive(Clone, PartialEq)]
pub struct CumulativeStats {
    /// not a stat, but avoids using a map in callers/callees
    pub hash: u32,
    pub sum: f64,
    pub sum_sqr: f64,
    pub min: f64,
    pub max: f64,
    pub count: u64,
    pub child_sum: f64,
}
