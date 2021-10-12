use crate::event_block::EventBlock;
use transit::prelude::*;

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

#[derive(Debug, TransitReflect)]
pub struct FloatMetricEvent {
    pub metric: &'static MetricDesc,
    pub value: f64,
}

impl InProcSerialize for FloatMetricEvent {}

declare_queue_struct!(
    struct MetricsMsgQueue<IntegerMetricEvent, FloatMetricEvent> {}
);

pub type MetricsBlock = EventBlock<MetricsMsgQueue>;
