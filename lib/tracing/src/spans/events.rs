use lgn_tracing_transit::prelude::*;

#[derive(Debug)]
pub struct ThreadSpanMetadata {
    pub lod: u32,
    pub name: &'static str,
    pub target: &'static str,
    pub module_path: &'static str,
    pub file: &'static str,
    pub line: u32,
}

#[derive(Debug, TransitReflect)]
pub struct BeginThreadSpanEvent {
    pub thread_span_desc: &'static ThreadSpanMetadata,
    pub time: i64,
}

impl InProcSerialize for BeginThreadSpanEvent {}

#[derive(Debug, TransitReflect)]
pub struct EndThreadSpanEvent {
    pub thread_span_desc: &'static ThreadSpanMetadata,
    pub time: i64,
}

impl InProcSerialize for EndThreadSpanEvent {}

#[derive(Debug, TransitReflect)]
pub struct ThreadSpanRecord {
    pub id: u64,
    pub name: *const u8,
    pub target: *const u8,
    pub module_path: *const u8,
    pub file: *const u8,
    pub line: u32,
    pub lod: u32,
}

impl InProcSerialize for ThreadSpanRecord {}
