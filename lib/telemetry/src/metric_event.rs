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
    pub time: u64,
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
    pub time: u64,
}

impl InProcSerialize for FloatMetricEvent {}
impl MetricEvent for FloatMetricEvent {
    fn get_metric(&self) -> &'static MetricDesc {
        self.metric
    }
}
