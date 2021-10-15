use bytes::Bytes;
use legion_ecs::prelude::*;

use log::{debug, warn};
use std::{cmp::min, sync::Arc};

use webrtc::data::data_channel::RTCDataChannel;

use legion_codec_api::{
    backends::openh264::encoder::{self, Encoder, FrameType},
    formats::{self, RBGYUVConverter},
};
use legion_mp4::old::Mp4Stream;
use legion_renderer::Renderer;
use legion_telemetry::prelude::*;
use legion_utils::memory::write_any;
use serde::Serialize;

fn record_frame_time_metric(microseconds: u64) {
    trace_scope!();
    static FRAME_TIME_METRIC: MetricDesc = MetricDesc {
        name: "Encoder Frame Time",
        unit: "us",
    };
    record_int_metric(&FRAME_TIME_METRIC, microseconds);
}

#[derive(PartialEq, Eq)]
struct Resolution {
    width: u32,
    height: u32,
}

impl Resolution {
    fn new(mut width: u32, mut height: u32) -> Self {
        // Ensure a minimum size for the resolution.
        if width < 16 {
            width = 16;
        }

        if height < 16 {
            height = 16;
        }

        Self {
            // Make sure width & height always are multiple of 2.
            width: width & !1,
            height: height & !1,
        }
    }
}

#[derive(Component)]
#[component(storage = "Table")]
pub struct VideoStream {
    video_data_channel: Arc<RTCDataChannel>,
    frame_id: i32,
    resolution: Resolution,
    pub hue: f32,
    pub speed: f32,
    renderer: Renderer,
    encoder: VideoStreamEncoder,
    elapsed_secs: f32,
}

impl VideoStream {
    pub fn new(video_data_channel: Arc<RTCDataChannel>) -> anyhow::Result<Self> {
        trace_scope!();
        let resolution = Resolution::new(1024, 768);
        let encoder = VideoStreamEncoder::new(&resolution)?;
        let renderer = Renderer::new(resolution.width, resolution.height);

        Ok(Self {
            video_data_channel,
            frame_id: 0,
            resolution,
            hue: 1.0,
            speed: 1.0,
            renderer,
            encoder,
            elapsed_secs: 0.0,
        })
    }

    pub(crate) fn resize(&mut self, width: u32, mut height: u32) {
        trace_scope!();
        // Make sure height is a multiple of 2.
        if height & 1 == 1 {
            height += 1;
        }

        let resolution = Resolution { width, height };

        if resolution != self.resolution {
            self.resolution = Resolution::new(width, height);

            // TODO: Fix this: this is probably bad but I wrote that just to test it.
            self.renderer = Renderer::new(self.resolution.width, self.resolution.height);
            self.encoder = VideoStreamEncoder::new(&self.resolution).unwrap();
        }
    }

    pub(crate) fn render(
        &mut self,
        delta_secs: f32,
    ) -> impl std::future::Future<Output = ()> + 'static {
        trace_scope!();
        let now = tokio::time::Instant::now();

        self.elapsed_secs += delta_secs * self.speed;

        self.renderer.render(
            self.frame_id as usize,
            self.elapsed_secs,
            hue2rgb_modulation(self.hue),
            &mut self.encoder.converter,
        );

        let chunks = self.encoder.encode();

        let elapsed = now.elapsed().as_micros() as u64;
        record_frame_time_metric(elapsed);
        let max_frame_time: u64 = 16_000;

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

struct VideoStreamEncoder {
    encoder: Encoder,
    converter: RBGYUVConverter,
    mp4: Mp4Stream,
    track_id: i32,
}

impl VideoStreamEncoder {
    fn new(resolution: &Resolution) -> anyhow::Result<Self> {
        trace_scope!();
        let config = encoder::EncoderConfig::new(resolution.width, resolution.height)
            .constant_sps(true)
            .max_fps(60.0)
            .skip_frame(false)
            .bitrate_bps(8_000_000);

        let encoder = encoder::Encoder::with_config(config)?;

        let converter =
            formats::RBGYUVConverter::new(resolution.width as usize, resolution.height as usize);

        let mut mp4 = Mp4Stream::new(60);
        #[allow(clippy::cast_possible_wrap)]
        let track_id = mp4
            .add_track(resolution.width as i32, resolution.height as i32)
            .unwrap();

        Ok(Self {
            encoder,
            converter,
            mp4,
            track_id,
        })
    }

    fn encode(&mut self) -> Vec<Bytes> {
        trace_scope!();
        let stream = self.encoder.encode(&self.converter).unwrap();

        for layer in &stream.layers {
            if !layer.is_video {
                for nalu in &layer.nal_units {
                    if nalu[4] == 103 {
                        self.mp4.set_sps(self.track_id, &nalu[4..]).unwrap();
                    } else if nalu[4] == 104 {
                        self.mp4.set_pps(self.track_id, &nalu[4..]).unwrap();
                    }
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
                    .add_frame(self.track_id, stream.frame_type == FrameType::IDR, &vec)
                    .unwrap();
            }
        }

        let data = self.mp4.get_content();
        let chunks = split_frame_in_chunks(data);
        self.mp4.clean();
        chunks
    }
}

#[derive(Serialize)]
struct ChunkHeader {
    pub chunk_index_in_frame: u8,
}

fn split_frame_in_chunks(data: &[u8]) -> Vec<Bytes> {
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

fn hue2rgb_modulation(hue: f32) -> (f32, f32, f32) {
    let rgb = hsl::HSL {
        h: f64::from(hue * 360.0),
        s: 1_f64,
        l: 0.5_f64,
    }
    .to_rgb();

    (
        f32::from(rgb.0) / 256.0,
        f32::from(rgb.1) / 256.0,
        f32::from(rgb.2) / 256.0,
    )
}
