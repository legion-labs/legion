use anyhow::Ok;
use anyhow::Result;
use lgn_analytics::find_stream;
use lgn_analytics::prelude::*;
use lgn_blob_storage::BlobStorage;
use lgn_telemetry_proto::analytics::BlockManifest;
use lgn_telemetry_proto::analytics::MetricBlockData;
use lgn_telemetry_proto::analytics::MetricBlockDesc;
use lgn_telemetry_proto::analytics::MetricBlockItem;
use lgn_telemetry_proto::analytics::MetricBlockRequest;
use lgn_telemetry_proto::analytics::MetricDataPoint;
use lgn_telemetry_proto::analytics::MetricDesc;
use lgn_telemetry_proto::analytics::MetricManifest;
use lgn_telemetry_proto::analytics::MetricRequestParams;
use lgn_telemetry_proto::analytics::ProcessMetricManifestReply;
use lgn_telemetry_proto::analytics::ProcessMetricReply;
use lgn_tracing_transit::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use xxhash_rust::const_xxh32::xxh32 as const_xxh32;

use crate::cache::DiskCache;

#[allow(clippy::cast_precision_loss)]
#[allow(dead_code)]
pub async fn get_process_metrics_time_range(
    sql: &mut sqlx::AnyConnection,
    process_id: &str,
) -> Result<(f64, f64)> {
    let mut min_ticks = i64::MAX;
    let mut max_ticks = i64::MIN;
    let process = find_process(sql, process_id).await?;
    for stream in find_process_metrics_streams(sql, process_id).await? {
        for block in find_stream_blocks(sql, &stream.stream_id).await? {
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

#[allow(clippy::cast_possible_wrap)]
fn reduce_lod(source: MetricBlockData, lod: u32) -> MetricBlockData {
    let merge_threshold = 100.0_f64.powi(lod as i32 - 2) / 10.0;
    let mut points: Vec<MetricDataPoint> = vec![];
    let mut max = f64::MIN;
    let mut acc = 0.0;
    for i in 0..source.points.len() - 1 {
        let point = &source.points[i];
        max = f64::max(point.value, max);
        let next_point = &source.points[i + 1];
        let delta = next_point.time_ms - point.time_ms;
        acc += delta;
        if acc > merge_threshold {
            points.push(MetricDataPoint {
                time_ms: point.time_ms,
                value: max,
            });
            max = f64::MIN;
            acc = 0.0;
        }
    }

    MetricBlockData {
        block_id: source.block_id,
        lod,
        points,
    }
}

fn get_lod_block_key(block_id: &str, metric_name: &str, lod: u32) -> String {
    [
        &const_xxh32(metric_name.as_bytes(), 0).to_string(),
        block_id,
        &lod.to_string(),
    ]
    .join("_")
}

fn get_request_cache_key(block_id: &str, params: &MetricRequestParams) -> String {
    get_lod_block_key(block_id, &params.metric_name, params.lod)
}

pub struct MetricHandler {
    blob_storage: Arc<dyn BlobStorage>,
    pool: sqlx::any::AnyPool,
    cache: Arc<DiskCache>,
}

impl MetricHandler {
    pub fn new(
        blob_storage: Arc<dyn BlobStorage>,
        cache: Arc<DiskCache>,
        pool: sqlx::any::AnyPool,
    ) -> Self {
        Self {
            blob_storage,
            pool,
            cache,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    pub async fn list_process_metrics(
        &self,
        process_id: &str,
    ) -> Result<ProcessMetricManifestReply> {
        let mut sql = self.pool.acquire().await?;
        let process = find_process(&mut sql, process_id).await?;
        let inv_tsc_frequency = 1000.0 / process.tsc_frequency as f64;
        let mut blocks = vec![];
        for stream in find_process_metrics_streams(&mut sql, process_id).await? {
            for block in find_stream_blocks(&mut sql, &stream.stream_id).await? {
                let begin_tick_offset = block.begin_ticks - process.start_ticks;
                let end_tick_offset = block.end_ticks - process.start_ticks;
                let block_desc = MetricBlockDesc {
                    block_id: block.block_id.clone(),
                    begin_ticks: begin_tick_offset,
                    end_ticks: end_tick_offset,
                    begin_time_ms: begin_tick_offset as f64 * inv_tsc_frequency,
                    end_time_ms: end_tick_offset as f64 * inv_tsc_frequency,
                    stream_id: stream.stream_id.clone(),
                };
                let payload =
                    fetch_block_payload(&mut sql, self.blob_storage.clone(), block.block_id)
                        .await?;
                let mut block_manifest = BlockManifest {
                    desc: Some(block_desc),
                    metrics: vec![],
                };
                let mut metrics = HashMap::<String, MetricDesc>::new();
                parse_block(&stream, &payload, |val| {
                    if let Value::Object(obj) = val {
                        let metric_desc = obj.get::<Object>("desc").unwrap();
                        let name = metric_desc.get_ref("name").unwrap().as_str().unwrap();
                        metrics
                            .entry(name.to_owned())
                            .or_insert_with(|| MetricDesc {
                                name: name.to_owned(),
                                unit: metric_desc
                                    .get_ref("unit")
                                    .unwrap()
                                    .as_str()
                                    .unwrap()
                                    .to_string(),
                            });
                    }
                    true
                })?;
                block_manifest.metrics = metrics.values().cloned().collect();
                blocks.push(block_manifest);
            }
        }

        let mut manifests = HashMap::new();

        for block in blocks {
            for metric in block.metrics {
                let manifest =
                    manifests
                        .entry(metric.name.clone())
                        .or_insert_with(|| MetricManifest {
                            name: metric.name,
                            unit: metric.unit,
                            blocks: vec![],
                        });

                if let Some(ref desc) = block.desc {
                    manifest.blocks.push(desc.clone());
                }
            }
        }

        Ok(ProcessMetricManifestReply {
            metrics: manifests.values().cloned().collect(),
            process_start_ticks: process.start_ticks,
            tsc_frequency: process.tsc_frequency,
        })
    }

    pub async fn fetch_metric(&self, request: MetricBlockRequest) -> Result<ProcessMetricReply> {
        let mut blocks = vec![];
        for block_item in request.blocks {
            if let Some(ref params) = request.params {
                blocks.push(self.get_block_lod(block_item, params.clone()).await?);
            }
        }
        Ok(ProcessMetricReply { blocks })
    }

    async fn get_block_lod(
        &self,
        block_item: MetricBlockItem,
        params: MetricRequestParams,
    ) -> Result<MetricBlockData> {
        let lod = params.lod;
        Ok(self
            .cache
            .get_or_put(
                &get_request_cache_key(&block_item.block_id, &params),
                async {
                    let raw = self
                        .cache
                        .get_or_put(
                            &get_lod_block_key(&block_item.block_id, &params.metric_name, 0),
                            async { Ok(self.get_raw_block(block_item, params).await?) },
                        )
                        .await?;
                    if lod > 0 {
                        Ok(reduce_lod(raw, lod))
                    } else {
                        Ok(raw)
                    }
                },
            )
            .await?)
    }

    #[allow(clippy::cast_precision_loss)]
    async fn get_raw_block(
        &self,
        block_item: MetricBlockItem,
        params: MetricRequestParams,
    ) -> Result<MetricBlockData> {
        let inv_tsc_frequency = 1000.0 / params.tsc_frequency as f64;
        let mut metric_block_data = MetricBlockData {
            block_id: block_item.block_id.clone(),
            lod: 0,
            points: vec![],
        };
        let mut connection = self.pool.acquire().await?;
        let stream = find_stream(&mut connection, &block_item.stream_id).await?;
        let payload = fetch_block_payload(
            &mut connection,
            self.blob_storage.clone(),
            block_item.block_id.clone(),
        )
        .await?;
        parse_block(&stream, &payload, |val| {
            if let Value::Object(obj) = val {
                let metric_desc = obj.get::<Object>("desc").unwrap();
                let name = metric_desc.get_ref("name").unwrap().as_str().unwrap();
                if name == params.metric_name {
                    let time = obj.get::<i64>("time").unwrap();
                    let time_ms = (time - params.process_start_ticks) as f64 * inv_tsc_frequency;
                    let value = obj.get::<u64>("value").unwrap();
                    metric_block_data.points.push(MetricDataPoint {
                        time_ms,
                        value: value as f64,
                    });
                }
            }
            true
        })?;

        Ok(metric_block_data)
    }
}
