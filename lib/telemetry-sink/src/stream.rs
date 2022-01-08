use std::collections::HashSet;

use anyhow::Result;
use lgn_telemetry_proto::compress;
use lgn_telemetry_proto::telemetry::Block as EncodedBlock;
use lgn_tracing::event_block::TelemetryBlock;
use lgn_tracing::{
    LogBlock, LogDepsQueue, LogMsgQueueAny, MetricEvent, MetricsBlock, MetricsDepsQueue,
    MetricsMsgQueueAny, ReferencedMetricDesc, ReferencedScope, ScopeDesc, ThreadBlock,
    ThreadDepsQueue, ThreadEventQueueAny,
};
use lgn_transit::{HeterogeneousQueue, StaticString};

pub trait StreamBlock {
    fn encode(&self) -> Result<EncodedBlock>;
}

impl StreamBlock for LogBlock {
    #[allow(clippy::cast_possible_wrap)]
    fn encode(&self) -> Result<EncodedBlock> {
        let block_id = uuid::Uuid::new_v4().to_string();
        let end = self.end.as_ref().unwrap();

        let mut deps = LogDepsQueue::new(1024 * 1024);
        let mut recorded_deps = HashSet::new();
        for x in self.events.iter() {
            match x {
                LogMsgQueueAny::LogMsgEvent(evt) => {
                    if recorded_deps.insert(evt.msg as u64) {
                        deps.push(StaticString {
                            len: evt.msg_len,
                            ptr: evt.msg,
                        });
                    }
                }
                LogMsgQueueAny::LogDynMsgEvent(_evt) => {}
            }
        }

        let payload = lgn_telemetry_proto::telemetry::BlockPayload {
            dependencies: compress(deps.as_bytes())?,
            objects: compress(self.events.as_bytes())?,
        };

        Ok(EncodedBlock {
            stream_id: self.stream_id.clone(),
            block_id,
            begin_time: self
                .begin
                .time
                .to_rfc3339_opts(chrono::SecondsFormat::Nanos, false),
            begin_ticks: self.begin.ticks,
            end_time: end
                .time
                .to_rfc3339_opts(chrono::SecondsFormat::Nanos, false),
            end_ticks: end.ticks,
            payload: Some(payload),
            nb_objects: self.nb_objects() as i32,
        })
    }
}

fn record_metric_event_dependencies<T: MetricEvent>(
    evt: &T,
    recorded_deps: &mut HashSet<u64>,
    deps: &mut MetricsDepsQueue,
) {
    let metric = evt.get_metric();
    let metric_ptr = std::ptr::addr_of!(*metric) as u64;
    if recorded_deps.insert(metric_ptr) {
        let name = StaticString::from(metric.name);
        if recorded_deps.insert(name.ptr as u64) {
            deps.push(name);
        }
        let unit = StaticString::from(metric.unit);
        if recorded_deps.insert(unit.ptr as u64) {
            deps.push(unit);
        }
        deps.push(ReferencedMetricDesc {
            id: metric_ptr,
            name: metric.name.as_ptr(),
            unit: metric.unit.as_ptr(),
        });
    }
}

impl StreamBlock for MetricsBlock {
    #[allow(clippy::cast_possible_wrap)]
    fn encode(&self) -> Result<EncodedBlock> {
        let block_id = uuid::Uuid::new_v4().to_string();
        let end = self.end.as_ref().unwrap();

        let mut deps = MetricsDepsQueue::new(1024 * 1024);
        let mut recorded_deps = HashSet::new();
        for x in self.events.iter() {
            match x {
                MetricsMsgQueueAny::IntegerMetricEvent(evt) => {
                    record_metric_event_dependencies(&evt, &mut recorded_deps, &mut deps);
                }
                MetricsMsgQueueAny::FloatMetricEvent(evt) => {
                    record_metric_event_dependencies(&evt, &mut recorded_deps, &mut deps);
                }
            }
        }

        let payload = lgn_telemetry_proto::telemetry::BlockPayload {
            dependencies: compress(deps.as_bytes())?,
            objects: compress(self.events.as_bytes())?,
        };

        Ok(EncodedBlock {
            stream_id: self.stream_id.clone(),
            block_id,
            begin_time: self
                .begin
                .time
                .to_rfc3339_opts(chrono::SecondsFormat::Nanos, false),
            begin_ticks: self.begin.ticks,
            end_time: end
                .time
                .to_rfc3339_opts(chrono::SecondsFormat::Nanos, false),
            end_ticks: end.ticks,
            payload: Some(payload),
            nb_objects: self.nb_objects() as i32,
        })
    }
}

fn record_scope_event_dependencies(
    evt_desc: &'static ScopeDesc,
    recorded_deps: &mut HashSet<u64>,
    deps: &mut ThreadDepsQueue,
) {
    let ptr = evt_desc as *const _ as u64;
    if recorded_deps.insert(ptr) {
        let name = StaticString::from(evt_desc.name);
        if recorded_deps.insert(name.ptr as u64) {
            deps.push(name);
        }
        let filename = StaticString::from(evt_desc.filename);
        if recorded_deps.insert(filename.ptr as u64) {
            deps.push(filename);
        }
        deps.push(ReferencedScope {
            id: ptr,
            name: evt_desc.name.as_ptr(),
            filename: evt_desc.filename.as_ptr(),
            line: evt_desc.line,
        });
    }
}

impl StreamBlock for ThreadBlock {
    #[allow(clippy::cast_possible_wrap)]
    fn encode(&self) -> Result<EncodedBlock> {
        let block_id = uuid::Uuid::new_v4().to_string();
        let end = self.end.as_ref().unwrap();

        let mut deps = ThreadDepsQueue::new(1024 * 1024);
        let mut recorded_deps = HashSet::new();
        for x in self.events.iter() {
            match x {
                ThreadEventQueueAny::BeginScopeEvent(evt) => {
                    record_scope_event_dependencies(evt.scope, &mut recorded_deps, &mut deps);
                }
                ThreadEventQueueAny::EndScopeEvent(evt) => {
                    record_scope_event_dependencies(evt.scope, &mut recorded_deps, &mut deps);
                }
            }
        }

        let payload = lgn_telemetry_proto::telemetry::BlockPayload {
            dependencies: compress(deps.as_bytes())?,
            objects: compress(self.events.as_bytes())?,
        };

        Ok(EncodedBlock {
            stream_id: self.stream_id.clone(),
            block_id,
            begin_time: self
                .begin
                .time
                .to_rfc3339_opts(chrono::SecondsFormat::Nanos, false),
            begin_ticks: self.begin.ticks,
            end_time: end
                .time
                .to_rfc3339_opts(chrono::SecondsFormat::Nanos, false),
            end_ticks: end.ticks,
            payload: Some(payload),
            nb_objects: self.nb_objects() as i32,
        })
    }
}
