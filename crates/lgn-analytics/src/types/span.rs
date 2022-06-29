use std::collections::HashMap;

use super::ScopeDesc;

/// Span: represents a function call instance
#[derive(Clone, PartialEq, prost::Message)]
pub struct Span {
    #[prost(uint32, tag = "1")]
    pub scope_hash: u32,
    #[prost(double, tag = "2")]
    pub begin_ms: f64,
    #[prost(double, tag = "3")]
    pub end_ms: f64,
    ///\[0-255\] non-linear transformation of occupancy for spans
    #[prost(uint32, tag = "4")]
    pub alpha: u32,
}

/// one span track contains spans at one height of call stack
#[derive(Clone, PartialEq, prost::Message)]
pub struct SpanTrack {
    #[prost(message, repeated, tag = "1")]
    pub spans: Vec<Span>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct SpanBlockLod {
    #[prost(uint32, tag = "1")]
    pub lod_id: u32,
    #[prost(message, repeated, tag = "2")]
    pub tracks: Vec<SpanTrack>,
}

/// async spans are identified by a unique u64 monotonically increasing
#[derive(Debug, Clone, PartialEq)]
pub struct AsyncSpan {
    pub span_id: u64,
    pub scope_hash: u32,
    pub begin_ms: f64,
    pub end_ms: f64,
    pub alpha: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AsyncSpanTrack {
    pub spans: Vec<AsyncSpan>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct BlockSpansReply {
    #[prost(map = "uint32, message", tag = "1")]
    pub scopes: HashMap<u32, ScopeDesc>,
    #[prost(message, optional, tag = "2")]
    pub lod: Option<SpanBlockLod>,
    #[prost(string, tag = "3")]
    pub block_id: String,
    #[prost(double, tag = "4")]
    pub begin_ms: f64,
    #[prost(double, tag = "5")]
    pub end_ms: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AsyncSpanEvent {
    pub event_type: i32,
    pub span_id: u64,
    pub scope_hash: u32,
    pub time_ms: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockAsyncData {
    pub block_id: String,
    pub scopes: HashMap<u32, ScopeDesc>,
    pub events: Vec<AsyncSpanEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SpanEventType {
    Begin = 0,
    End = 1,
}
