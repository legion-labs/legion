use anyhow::Result;
use lgn_tracing_transit::prelude::*;

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

pub type MetricsBlock = EventBlock<MetricsMsgQueue>;
pub type MetricsStream = EventStream<MetricsBlock, MetricsDepsQueue>;
