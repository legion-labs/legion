#[derive(Clone, PartialEq, prost::Message)]
pub struct MetricBlockData {
    #[prost(message, repeated, tag = "1")]
    pub points: Vec<MetricDataPoint>,
    #[prost(uint32, tag = "2")]
    pub lod: u32,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct MetricDataPoint {
    #[prost(double, tag = "1")]
    pub time_ms: f64,
    #[prost(double, tag = "2")]
    pub value: f64,
}

#[derive(Clone, PartialEq)]
pub struct MetricBlockRequest {
    pub process_id: String,
    pub block_id: String,
    pub stream_id: String,
    pub metric_name: String,
    pub lod: u32,
}

#[derive(Clone, PartialEq)]
pub struct MetricBlockManifestRequest {
    pub process_id: String,
    pub block_id: String,
    pub stream_id: String,
}

#[derive(Clone, PartialEq)]
pub struct MetricBlockManifest {
    pub desc: Option<MetricBlockDesc>,
    pub metrics: Vec<MetricDesc>,
}

#[derive(Clone, PartialEq)]
pub struct MetricBlockDesc {
    pub block_id: String,
    pub stream_id: String,
    pub begin_time_ms: f64,
    pub begin_ticks: i64,
    pub end_time_ms: f64,
    pub end_ticks: i64,
}

#[derive(Clone, PartialEq)]
pub struct MetricDesc {
    pub name: String,
    pub unit: String,
}
