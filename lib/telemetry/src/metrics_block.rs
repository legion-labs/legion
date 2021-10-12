use crate::{compress, event_block::EventBlock, EncodedBlock, EventStream, StreamBlock};
use anyhow::Result;
use std::collections::HashSet;
use transit::prelude::*;

pub trait MetricEvent {
    fn get_metric(&self) -> &'static MetricDesc;
}

#[derive(Debug)]
pub struct MetricDesc {
    pub name: &'static str,
    pub unit: &'static str,
}

#[derive(Debug, TransitReflect)]
pub struct IntegerMetricEvent {
    pub metric: &'static MetricDesc,
    pub value: u64,
}

impl InProcSerialize for IntegerMetricEvent {}
impl MetricEvent for IntegerMetricEvent {
    fn get_metric(&self) -> &'static MetricDesc {
        self.metric
    }
}

#[derive(Debug, TransitReflect)]
pub struct FloatMetricEvent {
    pub metric: &'static MetricDesc,
    pub value: f64,
}

impl InProcSerialize for FloatMetricEvent {}
impl MetricEvent for FloatMetricEvent {
    fn get_metric(&self) -> &'static MetricDesc {
        self.metric
    }
}

declare_queue_struct!(
    struct MetricsMsgQueue<IntegerMetricEvent, FloatMetricEvent> {}
);

#[derive(Debug, TransitReflect)]
pub struct ReferencedMetricDesc {
    pub id: u64,
    pub name: *const u8,
    pub unit: *const u8,
}

impl InProcSerialize for ReferencedMetricDesc {}

declare_queue_struct!(
    struct MetricsDepsQueue<StaticString, ReferencedMetricDesc> {}
);

pub type MetricsBlock = EventBlock<MetricsMsgQueue>;
pub type MetricsStream = EventStream<MetricsBlock, MetricsDepsQueue>;

fn record_metric_event_dependencies<T: MetricEvent>(
    evt: &T,
    recorded_deps: &mut HashSet<u64>,
    deps: &mut MetricsDepsQueue,
) {
    let metric = evt.get_metric();
    let metric_ptr = std::ptr::addr_of!(metric) as u64;
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

        let payload = legion_telemetry_proto::ingestion::BlockPayload {
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
        })
    }
}
