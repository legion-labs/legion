use std::collections::HashMap;

use super::ScopeDesc;

/// Span: represents a function call instance
#[derive(Clone, PartialEq)]
pub struct Span {
    pub scope_hash: u32,
    pub begin_ms: f64,
    pub end_ms: f64,
    ///\[0-255\] non-linear transformation of occupancy for spans
    pub alpha: u32,
}

/// one span track contains spans at one height of call stack
#[derive(Clone, PartialEq)]
pub struct SpanTrack {
    pub spans: ::prost::alloc::vec::Vec<Span>,
}

#[derive(Clone, PartialEq)]
pub struct SpanBlockLod {
    pub lod_id: u32,
    pub tracks: Vec<SpanTrack>,
}

/// async spans are identified by a unique u64 monotonically increasing
#[derive(Clone, PartialEq)]
pub struct AsyncSpan {
    pub span_id: u64,
    pub scope_hash: u32,
    pub begin_ms: f64,
    pub end_ms: f64,
    pub alpha: u32,
}

#[derive(Clone, PartialEq)]
pub struct AsyncSpanTrack {
    pub spans: Vec<AsyncSpan>,
}

#[derive(Clone, PartialEq)]
pub struct BlockSpansReply {
    pub scopes: HashMap<u32, ScopeDesc>,
    pub lod: Option<SpanBlockLod>,
    pub block_id: String,
    pub begin_ms: f64,
    pub end_ms: f64,
}

#[derive(Clone, PartialEq)]
pub struct AsyncSpanEvent {
    pub event_type: i32,
    pub span_id: u64,
    pub scope_hash: u32,
    pub time_ms: f64,
}

#[derive(Clone, PartialEq)]
pub struct BlockAsyncData {
    pub block_id: String,
    pub scopes: HashMap<u32, ScopeDesc>,
    pub events: Vec<AsyncSpanEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SpanEventType {
    Begin = 0,
    End = 1,
}
