use std::collections::HashSet;

use lgn_tracing_transit::prelude::*;

use crate::event_block::ExtractDeps;
use crate::prelude::*;
use crate::{event_block::EventBlock, EventStream};

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

impl ExtractDeps for MetricsMsgQueue {
    type DepsQueue = MetricsDepsQueue;

    fn extract(&self) -> Self::DepsQueue {
        let mut deps = MetricsDepsQueue::new(1024 * 1024);
        let mut recorded_deps = HashSet::new();
        for x in self.iter() {
            match x {
                MetricsMsgQueueAny::IntegerMetricEvent(evt) => {
                    record_metric_event_dependencies(&evt, &mut recorded_deps, &mut deps);
                }
                MetricsMsgQueueAny::FloatMetricEvent(evt) => {
                    record_metric_event_dependencies(&evt, &mut recorded_deps, &mut deps);
                }
            }
        }
        deps
    }
}

pub type MetricsBlock = EventBlock<MetricsMsgQueue>;
pub type MetricsStream = EventStream<MetricsBlock>;
