use std::{cmp::min, io::Cursor, sync::Arc};

use bytes::Bytes;
use legion_codec_api::{
    backends::openh264::encoder::{self, Encoder},
    formats::{self, RBGYUVConverter},
};
use legion_ecs::prelude::*;
use legion_graphics_api::prelude::*;
use legion_mp4::{AvcConfig, MediaConfig, Mp4Config, Mp4Stream};
use legion_presenter::offscreen_helper::{self, Resolution};
use legion_renderer::{
    components::{RenderSurface, RenderSurfaceExtents},
    Presenter, RenderContext, Renderer,
};
use legion_telemetry::prelude::*;
use legion_utils::{memory::write_any, setting_get_or};
use log::{debug, warn};
use serde::Serialize;
use webrtc::data::data_channel::RTCDataChannel;

fn record_frame_time_metric(microseconds: u64) {
    trace_scope!();
    static FRAME_TIME_METRIC: MetricDesc = MetricDesc {
        name: "Video Stream Frame Time",
        unit: "us",
    };
    record_int_metric(&FRAME_TIME_METRIC, microseconds);
}

#[derive(Component)]
#[component(storage = "Table")]
pub struct VideoStream {
    video_data_channel: Arc<RTCDataChannel>,
    frame_id: i32,
    encoder: VideoStreamEncoder,
    offscreen_helper: offscreen_helper::OffscreenHelper,
}

impl VideoStream {
    pub fn new(
        renderer: &Renderer,
        resolution: Resolution,
        video_data_channel: Arc<RTCDataChannel>,
    ) -> anyhow::Result<Self> {
        trace_scope!();

        let device_context = renderer.device_context();
        let graphics_queue = renderer.queue(QueueType::Graphics);
        let encoder = VideoStreamEncoder::new(resolution)?;
        let offscreen_helper =
            offscreen_helper::OffscreenHelper::new(device_context, &graphics_queue, resolution)?;

        Ok(Self {
            video_data_channel,
            frame_id: 0,
            encoder,
            offscreen_helper,
        })
    }

    pub(crate) fn resize(
        &mut self,
        renderer: &Renderer,
        extents: RenderSurfaceExtents,
    ) -> anyhow::Result<()> {
        trace_scope!();
        let device_context = renderer.device_context();
        let resolution = Resolution::new(extents.width(), extents.height());
        if self.offscreen_helper.resize(device_context, resolution)? {
            self.encoder = VideoStreamEncoder::new(resolution)?;
        }
        Ok(())
    }

    fn record_frame_id_metric(&self) {
        static FRAME_ID_RENDERED: MetricDesc = MetricDesc {
            name: "Frame ID begin present",
            unit: "",
        };
        record_int_metric(&FRAME_ID_RENDERED, self.frame_id as u64);
    }

    pub(crate) fn present<'renderer>(
        &mut self,
        render_context: &mut RenderContext<'renderer>,
        render_surface: &mut RenderSurface,
    ) -> impl std::future::Future<Output = ()> + 'static {
        trace_scope!();
        self.record_frame_id_metric();
        let now = tokio::time::Instant::now();

        //
        // Render
        //
        {
            self.offscreen_helper
                .present(
                    render_context,
                    render_surface,
                    |rgba: &[u8], row_pitch: usize| {
                        self.encoder.converter.convert_rgba(rgba, row_pitch);
                    },
                )
                .unwrap();
        }

        let chunks = self.encoder.encode(self.frame_id);

        let elapsed = now.elapsed().as_micros() as u64;
        record_frame_time_metric(elapsed);
        let max_frame_time: u64 = setting_get_or!("streamer.max_frame_time", 16_000u64);

        if elapsed >= max_frame_time {
            warn!(
                "stream: frame {:?} took {}ms",
                self.frame_id,
                elapsed / 1000
            );
        }

        let video_data_channel = Arc::clone(&self.video_data_channel);
        let frame_id = self.frame_id;

        self.frame_id += 1;

        async move {
            for (i, data) in chunks.iter().enumerate() {
                #[allow(clippy::redundant_else)]
                if let Err(err) = video_data_channel.send(data).await {
                    warn!(
                        "Failed to send frame {}-{} ({} bytes): streaming will stop: {}",
                        frame_id,
                        i,
                        data.len(),
                        err.to_string(),
                    );

                    return;
                } else {
                    debug!("Sent frame {}-{} ({} bytes).", frame_id, i, data.len());
                }
            }
        }
    }
}

impl Presenter for VideoStream {
    fn resize(&mut self, renderer: &Renderer, extents: RenderSurfaceExtents) {
        self.resize(renderer, extents).unwrap();
    }
    fn present<'renderer>(
        &mut self,
        render_context: &mut RenderContext<'renderer>,
        render_surface: &mut RenderSurface,
        async_rt: &mut legion_async::TokioAsyncRuntime,
    ) {
        async_rt.start_detached(self.present(render_context, render_surface));
    }
}

struct VideoStreamEncoder {
    encoder: Encoder,
    converter: RBGYUVConverter,
    resolution: Resolution,
    mp4: Mp4Stream,
    writer: Cursor<Vec<u8>>,
    write_index: bool,
}

impl VideoStreamEncoder {
    fn new(resolution: Resolution) -> anyhow::Result<Self> {
        trace_scope!();
        let config = encoder::EncoderConfig::new(resolution.width(), resolution.height())
            .constant_sps(true)
            .max_fps(60.0)
            .skip_frame(false)
            .bitrate_bps(8_000_000);

        let encoder = encoder::Encoder::with_config(config)?;

        let converter = formats::RBGYUVConverter::new(
            resolution.width() as usize,
            resolution.height() as usize,
        );

        let mut writer = Cursor::new(Vec::<u8>::new());
        let mp4 = Mp4Stream::write_start(
            &Mp4Config {
                major_brand: b"mp42".into(),
                minor_version: 0,
                compatible_brands: vec![b"mp42".into(), b"isom".into()],
                timescale: 1000,
            },
            60,
            &mut writer,
        )?;

        Ok(Self {
            encoder,
            converter,
            resolution,
            mp4,
            writer,
            write_index: true,
        })
    }

    fn encode(&mut self, frame_id: i32) -> Vec<Bytes> {
        trace_scope!();
        self.encoder.force_intra_frame(true);
        let stream = self.encoder.encode(&self.converter).unwrap();

        for layer in &stream.layers {
            if !layer.is_video {
                let mut sps: &[u8] = &[];
                let mut pps: &[u8] = &[];
                for nalu in &layer.nal_units {
                    if nalu[4] == 103 {
                        sps = &nalu[4..];
                    } else if nalu[4] == 104 {
                        pps = &nalu[4..];
                    }
                }
                if self.write_index {
                    self.mp4
                        .write_index(
                            &MediaConfig::AvcConfig(AvcConfig {
                                width: self.resolution.width().try_into().unwrap(),
                                height: self.resolution.height().try_into().unwrap(),
                                seq_param_set: sps.into(),
                                pic_param_set: pps.into(),
                            })
                            .into(),
                            &mut self.writer,
                        )
                        .unwrap();
                    self.write_index = false;
                }
                continue;
            }

            for nalu in &layer.nal_units {
                let size = nalu.len() - 4;
                let mut vec = vec![];
                vec.extend_from_slice(nalu);
                vec[0] = (size >> 24) as u8;
                vec[1] = ((size >> 16) & 0xFF) as u8;
                vec[2] = ((size >> 8) & 0xFF) as u8;
                vec[3] = (size & 0xFF) as u8;

                self.mp4
                    .write_sample(
                        stream.frame_type == encoder::FrameType::IDR,
                        &vec,
                        &mut self.writer,
                    )
                    .unwrap();
            }
        }

        let chunks = split_frame_in_chunks(self.writer.get_ref(), frame_id);
        self.writer.get_mut().clear();
        self.writer.set_position(0);
        chunks
    }
}

#[derive(Serialize)]
struct ChunkHeader {
    pub chunk_index_in_frame: u8,
    pub frame_id: i32,
}

fn split_frame_in_chunks(data: &[u8], frame_id: i32) -> Vec<Bytes> {
    let max_chunk_size = 65536;
    let mut chunks = vec![];
    chunks.reserve((data.len() / max_chunk_size) + 2);

    let mut current_chunk: Vec<u8> = vec![];
    current_chunk.reserve(max_chunk_size);

    let mut current_data_index = 0;
    let mut chunk_index_in_frame: u8 = 0;
    while current_data_index < data.len() {
        current_chunk.clear();
        let header = serde_json::to_string(&ChunkHeader {
            chunk_index_in_frame,
            frame_id,
        })
        .unwrap();
        let header_payload_len: u16 = header.len() as u16;
        let header_size = std::mem::size_of::<u16>() as u16 + header_payload_len;
        write_any(&mut current_chunk, &header_payload_len);
        current_chunk.extend_from_slice(header.as_bytes());
        let end_chunk = min(
            current_data_index + max_chunk_size - header_size as usize,
            data.len(),
        );
        let chunk_data_slice = &data[current_data_index..end_chunk];
        current_chunk.extend_from_slice(chunk_data_slice);
        chunks.push(bytes::Bytes::copy_from_slice(&current_chunk));
        current_data_index = end_chunk;
        chunk_index_in_frame += 1;
    }

    chunks
}
