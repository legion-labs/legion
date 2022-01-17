use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use lgn_analytics::prelude::*;
use lgn_telemetry_proto::analytics::MetricDataPoint;
use lgn_telemetry_proto::analytics::MetricDesc;
use lgn_tracing_transit::prelude::*;

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
    data_path: &Path,
    process_id: &str,
) -> Result<Vec<MetricDesc>> {
    let mut metrics = HashMap::<String, MetricDesc>::new();
    for_each_process_metric(connection, data_path, process_id, |metric_instance| {
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

#[allow(clippy::cast_precision_loss)]
pub async fn fetch_process_metric(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    process_id: &str,
    metric_name: &str,
    _unit: &str,
    begin_ms: f64,
    end_ms: f64,
) -> Result<Vec<MetricDataPoint>> {
    let mut points = vec![];
    let process = find_process(connection, process_id).await?;
    let inv_tsc_frequency = 1000.0 / process.tsc_frequency as f64;
    for_each_process_metric(connection, data_path, process_id, |metric_instance| {
        let metric_desc = metric_instance.get::<Object>("desc").unwrap();
        let name = metric_desc.get_ref("name").unwrap().as_str().unwrap();
        if name == metric_name {
            let time = metric_instance.get::<i64>("time").unwrap();
            let time_ms = (time - process.start_ticks) as f64 * inv_tsc_frequency;
            //todo: test in ticks, convert to ms after
            if time_ms >= begin_ms && time_ms <= end_ms {
                let value = metric_instance.get::<u64>("value").unwrap();
                points.push(MetricDataPoint {
                    time_ms,
                    value: value as f64,
                });
            }
        }
    })
    .await?;
    Ok(points)
}
