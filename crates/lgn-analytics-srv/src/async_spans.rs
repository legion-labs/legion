use anyhow::Result;
use lgn_telemetry_proto::analytics::AsyncSpanEvent;
use lgn_telemetry_proto::analytics::BlockAsyncData;
use lgn_telemetry_proto::analytics::SpanEventType;
use lgn_telemetry_proto::analytics::{
    AsyncSpan, AsyncSpanTrack, AsyncSpansReply, BlockAsyncEventsStatReply,
};
use lgn_tracing::prelude::*;
use std::collections::HashMap;

use crate::{call_tree_store::CallTreeStore, scope::ScopeHashMap};

#[allow(clippy::cast_precision_loss)]
#[span_fn]
pub async fn compute_block_async_stats(
    call_tree_store: &CallTreeStore,
    block_id: String,
) -> Result<BlockAsyncEventsStatReply> {
    let async_data = call_tree_store.get_block_async_data(&block_id).await?;
    let (begin_ms, end_ms) = if async_data.events.is_empty() {
        (f64::MAX, f64::MIN)
    } else {
        (
            async_data.events[0].time_ms,
            async_data.events[async_data.events.len() - 1].time_ms,
        )
    };
    Ok(BlockAsyncEventsStatReply {
        block_id,
        begin_ms,
        end_ms,
        nb_events: async_data.events.len() as u64,
    })
}

fn ranges_overlap(begin_a: f64, end_a: f64, begin_b: f64, end_b: f64) -> bool {
    begin_a <= end_b && begin_b <= end_a
}

struct AsyncSpanBuilder {
    begin_section_ms: f64,
    end_section_ms: f64,
    unmatched_events: HashMap<u64, AsyncSpanEvent>,
    complete_spans: Vec<AsyncSpan>,
    scopes: ScopeHashMap,
}

impl AsyncSpanBuilder {
    fn new(begin_section_ms: f64, end_section_ms: f64) -> Self {
        Self {
            begin_section_ms,
            end_section_ms,
            unmatched_events: HashMap::new(),
            complete_spans: Vec::new(),
            scopes: ScopeHashMap::new(),
        }
    }

    fn record_span(&mut self, span_id: u64, begin_ms: f64, end_ms: f64, scope_hash: u32) {
        if ranges_overlap(self.begin_section_ms, self.end_section_ms, begin_ms, end_ms) {
            self.complete_spans.push(AsyncSpan {
                span_id,
                scope_hash,
                begin_ms: begin_ms.max(self.begin_section_ms),
                end_ms: end_ms.min(self.end_section_ms),
                alpha: 255,
            });
        }
    }

    #[span_fn]
    fn finish(mut self) -> (Vec<AsyncSpan>, ScopeHashMap) {
        let mut events = HashMap::new();
        std::mem::swap(&mut events, &mut self.unmatched_events);
        for (_id, evt) in events {
            match SpanEventType::from_i32(evt.event_type) {
                Some(SpanEventType::Begin) => {
                    self.record_span(
                        evt.span_id,
                        evt.time_ms,
                        self.end_section_ms,
                        evt.scope_hash,
                    );
                }
                Some(SpanEventType::End) => {
                    self.record_span(
                        evt.span_id,
                        self.begin_section_ms,
                        evt.time_ms,
                        evt.scope_hash,
                    );
                }
                None => {
                    warn!("unknown event type {}", evt.event_type);
                }
            }
        }
        (self.complete_spans, self.scopes)
    }

    pub fn process(&mut self, data: BlockAsyncData) -> Result<()> {
        for (k, v) in data.scopes {
            self.scopes.insert(k, v);
        }
        for evt in data.events {
            match SpanEventType::from_i32(evt.event_type) {
                Some(SpanEventType::Begin) => {
                    if let Some(matched) = self.unmatched_events.remove(&evt.span_id) {
                        match SpanEventType::from_i32(matched.event_type) {
                            Some(SpanEventType::Begin) => {
                                anyhow::bail!(
                                    "duplicate begin event for span id {}: {:?}",
                                    evt.span_id,
                                    matched
                                );
                            }
                            Some(SpanEventType::End) => {
                                self.record_span(
                                    evt.span_id,
                                    evt.time_ms,
                                    matched.time_ms,
                                    evt.scope_hash,
                                );
                            }
                            None => {
                                warn!("unknown event type {}", matched.event_type);
                            }
                        }
                    } else {
                        self.unmatched_events.insert(evt.span_id, evt);
                    }
                }
                Some(SpanEventType::End) => {
                    if let Some(matched) = self.unmatched_events.remove(&evt.span_id) {
                        match SpanEventType::from_i32(matched.event_type) {
                            Some(SpanEventType::End) => {
                                anyhow::bail!(
                                    "duplicate end event for span id {}: {:?}",
                                    evt.span_id,
                                    matched
                                );
                            }
                            Some(SpanEventType::Begin) => {
                                self.record_span(
                                    evt.span_id,
                                    matched.time_ms,
                                    evt.time_ms,
                                    evt.scope_hash,
                                );
                            }
                            None => {
                                warn!("unknown event type {}", matched.event_type);
                            }
                        }
                    } else {
                        self.unmatched_events.insert(evt.span_id, evt);
                    }
                }
                None => {
                    warn!("unknown event type {}", evt.event_type);
                }
            }
        }
        Ok(())
    }
}

fn is_track_available(track: &[AsyncSpan], time: f64) -> bool {
    if let Some(last) = track.last() {
        last.end_ms <= time
    } else {
        true
    }
}

#[span_fn]
fn get_available_track(tracks: &mut Vec<AsyncSpanTrack>, time: f64) -> usize {
    for (index, track) in tracks.iter().enumerate() {
        if is_track_available(&track.spans, time) {
            return index;
        }
    }
    tracks.push(AsyncSpanTrack { spans: vec![] });
    tracks.len() - 1
}

#[span_fn]
fn layout_spans(spans: Vec<AsyncSpan>) -> Vec<AsyncSpanTrack> {
    let mut tracks = vec![];
    for span in spans {
        let index = get_available_track(&mut tracks, span.begin_ms);
        tracks[index].spans.push(span);
    }
    tracks
}

#[allow(clippy::cast_lossless)]
#[span_fn]
pub async fn compute_async_spans(
    call_tree_store: &CallTreeStore,
    section_sequence_number: i32,
    section_lod: u32,
    block_ids: Vec<String>,
) -> Result<AsyncSpansReply> {
    if section_lod != 0 {
        anyhow::bail!("async lods not implemented");
    }
    let section_width_ms = 1000.0;
    let begin_section_ms = section_sequence_number as f64 * section_width_ms;
    let end_section_ms = begin_section_ms + section_width_ms;
    if block_ids.is_empty() {
        return Ok(AsyncSpansReply {
            section_sequence_number,
            section_lod,
            tracks: vec![],
            scopes: ScopeHashMap::new(),
        });
    }
    let mut builder = AsyncSpanBuilder::new(begin_section_ms, end_section_ms);
    for block_id in &block_ids {
        let async_data = call_tree_store.get_block_async_data(block_id).await?;
        builder.process(async_data)?;
    }
    let (mut spans, scopes) = builder.finish();
    spans.sort_by(|a, b| a.span_id.partial_cmp(&b.span_id).unwrap());
    let tracks = layout_spans(spans);
    let reply = AsyncSpansReply {
        section_sequence_number,
        section_lod,
        tracks,
        scopes,
    };
    Ok(reply)
}
