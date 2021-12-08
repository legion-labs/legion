use anyhow::Result;
use lgn_analytics::prelude::*;
use lgn_telemetry_proto::analytics::MetricDesc;
use lgn_transit::prelude::*;
use std::collections::HashMap;
use std::path::Path;

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
        let metric_desc = metric_instance.get::<Object>("metric").unwrap();
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
