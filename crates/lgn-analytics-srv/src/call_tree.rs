use std::sync::Arc;

use anyhow::Result;
use lgn_analytics::prelude::*;
use lgn_analytics::time::ConvertTicks;
use lgn_blob_storage::BlobStorage;
use lgn_telemetry_proto::analytics::AsyncSpanEvent;
use lgn_telemetry_proto::analytics::BlockSpansReply;
use lgn_telemetry_proto::analytics::CallTree;
use lgn_telemetry_proto::analytics::CallTreeNode;
use lgn_telemetry_proto::analytics::ScopeDesc;
use lgn_telemetry_proto::analytics::Span;
use lgn_telemetry_proto::analytics::SpanBlockLod;
use lgn_telemetry_proto::analytics::SpanEventType;
use lgn_telemetry_proto::analytics::SpanTrack;
use lgn_tracing::prelude::*;

pub struct ProcessedThreadBlock {
    pub scopes: ScopeHashMap,
    pub call_tree_root: Option<CallTreeNode>,
    pub async_events: Vec<AsyncSpanEvent>,
}

struct CallTreeBuilder {
    ts_begin_block: i64,
    ts_end_block: i64,
    stack: Vec<CallTreeNode>,
    scopes: ScopeHashMap,
    convert_ticks: ConvertTicks,
    async_events: Vec<AsyncSpanEvent>,
}

impl CallTreeBuilder {
    pub fn new(ts_begin_block: i64, ts_end_block: i64, convert_ticks: ConvertTicks) -> Self {
        Self {
            ts_begin_block,
            ts_end_block,
            stack: Vec::new(),
            scopes: ScopeHashMap::new(),
            convert_ticks,
            async_events: Vec::new(),
        }
    }

    #[span_fn]
    pub fn finish(mut self) -> ProcessedThreadBlock {
        if self.stack.is_empty() {
            return ProcessedThreadBlock {
                scopes: ScopeHashMap::new(),
                call_tree_root: None,
                async_events: self.async_events,
            };
        }
        while self.stack.len() > 1 {
            let top = self.stack.pop().unwrap();
            let last_index = self.stack.len() - 1;
            let parent = &mut self.stack[last_index];
            parent.children.push(top);
        }
        assert_eq!(1, self.stack.len());
        ProcessedThreadBlock {
            scopes: self.scopes,
            call_tree_root: self.stack.pop(),
            async_events: self.async_events,
        }
    }

    fn add_child_to_top(&mut self, scope: CallTreeNode) {
        if let Some(mut top) = self.stack.pop() {
            top.children.push(scope);
            self.stack.push(top);
        } else {
            let new_root = CallTreeNode {
                hash: 0,
                begin_ms: self.convert_ticks.get_time(self.ts_begin_block),
                end_ms: self.convert_ticks.get_time(self.ts_end_block),
                children: vec![scope],
            };
            self.stack.push(new_root);
        }
    }

    fn record_scope_desc(&mut self, hash: u32, name: &str) {
        self.scopes.entry(hash).or_insert_with(|| ScopeDesc {
            name: name.to_owned(),
            filename: "".to_string(),
            line: 0,
            hash,
        });
    }
}

impl ThreadBlockProcessor for CallTreeBuilder {
    fn on_begin_thread_scope(
        &mut self,
        _scope: Arc<Object>,
        name: Arc<String>,
        ts: i64,
    ) -> Result<()> {
        let time = self.convert_ticks.get_time(ts);
        let hash = compute_scope_hash(&name);
        self.record_scope_desc(hash, &name);
        let scope = CallTreeNode {
            hash,
            begin_ms: time,
            end_ms: self.convert_ticks.get_time(self.ts_end_block),
            children: Vec::new(),
        };
        self.stack.push(scope);
        Ok(())
    }

    fn on_end_thread_scope(
        &mut self,
        _scope: Arc<Object>,
        name: Arc<String>,
        ts: i64,
    ) -> Result<()> {
        let time = self.convert_ticks.get_time(ts);
        let hash = compute_scope_hash(&name);
        if let Some(mut old_top) = self.stack.pop() {
            if old_top.hash == hash {
                old_top.end_ms = time;
                self.add_child_to_top(old_top);
            } else if old_top.hash == 0 {
                self.record_scope_desc(hash, &name);
                old_top.hash = hash;
                old_top.end_ms = time;
                self.add_child_to_top(old_top);
            } else {
                anyhow::bail!("top scope mismatch parsing thread block");
            }
        } else {
            self.record_scope_desc(hash, &name);
            let scope = CallTreeNode {
                hash,
                begin_ms: self.convert_ticks.get_time(self.ts_begin_block),
                end_ms: time,
                children: Vec::new(),
            };
            self.add_child_to_top(scope);
        }
        Ok(())
    }

    fn on_begin_async_scope(
        &mut self,
        span_id: u64,
        _scope: Arc<Object>,
        name: Arc<String>,
        ts: i64,
    ) -> Result<()> {
        let time_ms = self.convert_ticks.get_time(ts);
        let scope_hash = compute_scope_hash(&name);
        self.record_scope_desc(scope_hash, &name);
        self.async_events.push(AsyncSpanEvent {
            event_type: SpanEventType::Begin as i32,
            span_id,
            scope_hash,
            time_ms,
        });
        Ok(())
    }

    fn on_end_async_scope(
        &mut self,
        span_id: u64,
        _scope: Arc<Object>,
        name: Arc<String>,
        ts: i64,
    ) -> Result<()> {
        let time_ms = self.convert_ticks.get_time(ts);
        let scope_hash = compute_scope_hash(&name);
        self.record_scope_desc(scope_hash, &name);
        self.async_events.push(AsyncSpanEvent {
            event_type: SpanEventType::End as i32,
            span_id,
            scope_hash,
            time_ms,
        });
        Ok(())
    }
}

#[allow(clippy::cast_precision_loss)]
#[span_fn]
pub(crate) async fn process_thread_block(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    convert_ticks: ConvertTicks,
    stream: &lgn_telemetry_sink::StreamInfo,
    block_id: &str,
) -> Result<ProcessedThreadBlock> {
    let block = find_block(connection, block_id).await?;
    let mut builder = CallTreeBuilder::new(block.begin_ticks, block.end_ticks, convert_ticks);
    parse_thread_block(
        connection,
        blob_storage,
        stream,
        block_id.to_owned(),
        &mut builder,
    )
    .await?;
    Ok(builder.finish())
}

use lgn_tracing_transit::Object;

use crate::scope::compute_scope_hash;
use crate::scope::ScopeHashMap;
use crate::thread_block_processor::parse_thread_block;
use crate::thread_block_processor::ThreadBlockProcessor;

fn make_spans_from_tree(tree: &CallTreeNode, depth: u32, lod: &mut SpanBlockLod) {
    let span = Span {
        scope_hash: tree.hash,
        begin_ms: tree.begin_ms,
        end_ms: tree.end_ms,
        alpha: 255,
    };
    if lod.tracks.len() <= depth as usize {
        lod.tracks.push(SpanTrack { spans: vec![] });
    }
    assert!(lod.tracks.len() > depth as usize);
    lod.tracks[depth as usize].spans.push(span);
    for child in &tree.children {
        make_spans_from_tree(child, depth + 1, lod);
    }
}

#[span_fn]
pub(crate) fn compute_block_spans(tree: CallTree, block_id: &str) -> BlockSpansReply {
    if tree.root.is_none() {
        info!("empty call tree for block {}", block_id);
        return BlockSpansReply {
            scopes: ScopeHashMap::new(),
            lod: Some(SpanBlockLod {
                lod_id: 0,
                tracks: vec![],
            }),
            block_id: block_id.to_owned(),
            begin_ms: f64::MAX,
            end_ms: f64::MIN,
        };
    }

    let root = tree.root.unwrap();
    let mut begin_ms = root.begin_ms;
    let mut end_ms = root.end_ms;
    let mut lod = SpanBlockLod {
        lod_id: 0,
        tracks: vec![],
    };
    if root.hash == 0 {
        begin_ms = f64::MAX;
        end_ms = f64::MIN;
        for child in &root.children {
            begin_ms = begin_ms.min(child.begin_ms);
            end_ms = end_ms.max(child.end_ms);
            make_spans_from_tree(child, 0, &mut lod);
        }
    } else {
        make_spans_from_tree(&root, 0, &mut lod);
    }

    BlockSpansReply {
        scopes: tree.scopes,
        lod: Some(lod),
        block_id: block_id.to_owned(),
        begin_ms,
        end_ms,
    }
}

#[allow(clippy::cast_possible_wrap)]
#[span_fn]
pub(crate) fn reduce_lod(lod0: &SpanBlockLod, lod_id: u32) -> SpanBlockLod {
    let merge_threshold = 100.0_f64.powi(lod_id as i32 - 2) / 10.0;
    let mut tracks = vec![];
    for track in &lod0.tracks {
        let mut reduced_spans = vec![];
        let mut current_acc = track.spans[0].clone();
        let mut time_sum = current_acc.end_ms - current_acc.begin_ms;
        let mut index = 1;
        while index < track.spans.len() {
            let span = track.spans[index].clone();
            if span.end_ms - current_acc.begin_ms > merge_threshold {
                let nonlinear_occupancy =
                    (time_sum / (current_acc.end_ms - current_acc.begin_ms)).sqrt();
                current_acc.alpha = (nonlinear_occupancy * 255.0).floor() as u32;
                reduced_spans.push(current_acc);
                current_acc = span;
                time_sum = current_acc.end_ms - current_acc.begin_ms;
            } else {
                current_acc.scope_hash = 0;
                current_acc.end_ms = span.end_ms;
                time_sum += span.end_ms - span.begin_ms;
            }
            index += 1;
        }
        let nonlinear_occupancy = (time_sum / (current_acc.end_ms - current_acc.begin_ms)).sqrt();
        current_acc.alpha = (nonlinear_occupancy * 255.0).floor() as u32;
        reduced_spans.push(current_acc);
        tracks.push(SpanTrack {
            spans: reduced_spans,
        });
    }
    SpanBlockLod { lod_id, tracks }
}
