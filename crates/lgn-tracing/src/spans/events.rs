use lgn_tracing_transit::prelude::*;

use crate::Verbosity;

#[derive(Debug)]
pub struct SpanMetadata {
    pub lod: Verbosity,
    pub name: &'static str,
    pub target: &'static str,
    pub module_path: &'static str,
    pub file: &'static str,
    pub line: u32,
}

// SpanRecord is the serialized version of SpanMetadata
#[derive(Debug, TransitReflect)]
pub struct SpanRecord {
    pub id: u64,
    pub name: *const u8,
    pub target: *const u8,
    pub module_path: *const u8,
    pub file: *const u8,
    pub line: u32,
    pub lod: u32,
}

impl InProcSerialize for SpanRecord {}

//
// sync events
//
#[derive(Debug, TransitReflect)]
pub struct BeginThreadSpanEvent {
    pub thread_span_desc: &'static SpanMetadata,
    pub time: i64,
}

impl InProcSerialize for BeginThreadSpanEvent {}

#[derive(Debug, TransitReflect)]
pub struct EndThreadSpanEvent {
    pub thread_span_desc: &'static SpanMetadata,
    pub time: i64,
}

impl InProcSerialize for EndThreadSpanEvent {}

//
// async events
//
#[derive(Debug, TransitReflect)]
pub struct BeginAsyncSpanEvent {
    pub span_desc: &'static SpanMetadata,
    pub span_id: u64,
    pub time: i64,
}

impl InProcSerialize for BeginAsyncSpanEvent {}

#[derive(Debug, TransitReflect)]
pub struct EndAsyncSpanEvent {
    pub span_desc: &'static SpanMetadata,
    pub span_id: u64,
    pub time: i64,
}

impl InProcSerialize for EndAsyncSpanEvent {}
