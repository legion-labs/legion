use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Ok;
use anyhow::Result;
use lgn_analytics::prelude::*;
use lgn_blob_storage::BlobStorage;
use lgn_telemetry_proto::analytics::MetricDataPoint;
use lgn_telemetry_proto::analytics::MetricDesc;
use lgn_telemetry_proto::analytics::ProcessMetricReply;
use lgn_tracing_transit::prelude::*;
use xxhash_rust::const_xxh32::xxh32 as const_xxh32;

use crate::cache::DiskCache;

#[allow(clippy::cast_precision_loss)]
pub async fn get_process_metrics_time_range(
    connection: &mut sqlx::AnyConnection,
    process_id: &str,
) -> Result<(f64, f64)> {
    let mut min_ticks = i64::MAX;
    let mut max_ticks = i64::MIN;
    let process = find_process(connection, process_id).await?;
    for stream in find_process_metrics_streams(connection, process_id).await? {
        for block in find_stream_blocks(connection, &stream.stream_id).await? {
            let block_begin = block.begin_ticks - process.start_ticks;
            let block_end = block.end_ticks - process.start_ticks;
            min_ticks = std::cmp::min(min_ticks, block_begin);
            max_ticks = std::cmp::max(max_ticks, block_end);
        }
    }
    let inv_tsc_frequency = 1000.0 / process.tsc_frequency as f64;
    Ok((
        min_ticks as f64 * inv_tsc_frequency,
        max_ticks as f64 * inv_tsc_frequency,
    ))
}

pub async fn list_process_metrics(
    connection: &mut sqlx::AnyConnection,
    blob_storage: Arc<dyn BlobStorage>,
    process_id: &str,
) -> Result<Vec<MetricDesc>> {
    let mut metrics = HashMap::<String, MetricDesc>::new();
    for_each_process_metric(connection, blob_storage, process_id, |metric_instance| {
        let metric_desc = metric_instance.get::<Object>("desc").unwrap();
        let name = metric_desc.get_ref("name").unwrap().as_str().unwrap();
        let unit = metric_desc.get_ref("unit").unwrap().as_str().unwrap();
        metrics
            .entry(name.to_owned())
            .or_insert_with(|| MetricDesc {
                name: name.to_owned(),
                unit: unit.to_owned(),
            });
    })
    .await?;
    Ok(metrics.values().cloned().collect())
}

fn decimate_from_source(source: &ProcessMetricReply, lod: u32) -> ProcessMetricReply {
    let time_ticks = source.points.iter().map(|x| x.time_ms).collect::<Vec<_>>();
    let min_tick = time_ticks
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max_tick = time_ticks
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let time_span = max_tick - min_tick;

    let min_ticks = 2000;
    let max_ticks = source.points.len();
    let ticks_count = max_ticks as u32 / (lod.pow(2) + 1);
    let ticks_count = std::cmp::max(min_ticks, ticks_count);
    let tick_size = time_span / f64::from(ticks_count) as f64;

    let mut points: Vec<MetricDataPoint> = vec![];

    for tick in 0..ticks_count - 1 {
        let min = f64::from(tick) * tick_size;
        let max = f64::from(tick + 1) * tick_size;
        let mut value: f64 = f64::MIN;
        for point in &source.points {
            if (point.time_ms >= min && point.time_ms < max) && (value < point.value) {
                value = point.value;
            }
        }
        points.push(MetricDataPoint {
            value,
            time_ms: f64::from(tick) * (max - min),
        });
    }

    ProcessMetricReply { points, lod }
}

fn get_lod_cache_key(process_id: &str, metric_name: &str, lod: u32) -> String {
    [
        process_id,
        &const_xxh32(metric_name.as_bytes(), 0).to_string(),
        &lod.to_string(),
    ]
    .join("_")
}

pub struct MetricHandler {
    blob_storage: Arc<dyn BlobStorage>,
    pool: Arc<sqlx::any::AnyPool>,
    cache: DiskCache,
}

impl MetricHandler {
    pub async fn new(
        blob_storage: Arc<dyn BlobStorage>,
        pool: Arc<sqlx::any::AnyPool>,
    ) -> Result<Self> {
        Ok(Self {
            blob_storage,
            pool,
            cache: DiskCache::new().await?,
        })
    }

    pub async fn fetch_metric(
        &self,
        process_id: &str,
        metric_name: &str,
        _begin_ms: f64,
        _end_ms: f64,
        lod: u32,
    ) -> Result<ProcessMetricReply> {
        // For now we are not streaming the points so begin and max are not used.
        let requested_lod = self.get_metric_data(process_id, metric_name, lod).await?;
        Ok(requested_lod)
    }

    async fn get_metric_data(
        &self,
        process_id: &str,
        metric_name: &str,
        lod: u32,
    ) -> Result<ProcessMetricReply> {
        let key = get_lod_cache_key(process_id, metric_name, lod);
        let result = self
            .cache
            .get_or_put(&key, async {
                let lod0key = get_lod_cache_key(process_id, metric_name, 0);
                let lod0 = self
                    .cache
                    .get_or_put(&lod0key, async {
                        Ok(ProcessMetricReply {
                            lod: 0,
                            points: self.get_raw_metric_data(process_id, metric_name).await?,
                        })
                    })
                    .await?;

                if lod == 0 {
                    Ok(lod0)
                } else {
                    Ok(decimate_from_source(&lod0, lod))
                }
            })
            .await?;

        Ok(result)
    }

    #[allow(clippy::cast_precision_loss)]
    async fn get_raw_metric_data(
        &self,
        process_id: &str,
        metric_name: &str,
    ) -> Result<Vec<MetricDataPoint>> {
        let mut connection = self.pool.acquire().await?;
        let process = find_process(&mut connection, process_id).await?;
        let inv_tsc_frequency = 1000.0 / process.tsc_frequency as f64;
        let mut points: Vec<MetricDataPoint> = vec![];
        for_each_process_metric(
            &mut connection,
            Arc::clone(&self.blob_storage),
            process_id,
            |metric_instance| {
                let metric_desc = metric_instance.get::<Object>("desc").unwrap();
                let name = metric_desc.get_ref("name").unwrap().as_str().unwrap();
                if name == metric_name {
                    let time = metric_instance.get::<i64>("time").unwrap();
                    let time_ms = (time - process.start_ticks) as f64 * inv_tsc_frequency;
                    let value = metric_instance.get::<u64>("value").unwrap();
                    points.push(MetricDataPoint {
                        time_ms,
                        value: value as f64,
                    });
                }
            },
        )
        .await?;
        Ok(points)
    }
}
