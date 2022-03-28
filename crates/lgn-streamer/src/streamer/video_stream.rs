use std::{io::Cursor, sync::Arc};

use bytes::{BufMut, Bytes};
use lgn_async::TokioAsyncRuntimeHandle;
use lgn_codec_api::{
    backends::{
        nvenc::NvEncEncoderWrapper,
        openh264::encoder::{self, Encoder},
        CodecHardware::Nvidia,
        EncoderConfig,
    },
    encoder_work_queue::{EncoderWorkItem, EncoderWorkQueue},
    formats::YUVSource,
    VideoProcessor,
};
use lgn_ecs::prelude::*;
use lgn_graphics_api::DeviceContext;
use lgn_graphics_renderer::{
    components::{Presenter, RenderSurface, RenderSurfaceExtents},
    resources::PipelineManager,
    RenderContext,
};
use lgn_mp4::{AvcConfig, MediaConfig, Mp4Config, Mp4Stream};
use lgn_tracing::prelude::*;
use lgn_tracing::{debug, warn};
use serde::Serialize;
use webrtc::data_channel::{data_channel_state::RTCDataChannelState, RTCDataChannel};

use super::{Resolution, RgbToYuvConverter};

#[span_fn]
fn record_frame_time_metric(microseconds: u64) {
    imetric!("Video Stream Frame Time", "us", microseconds);
}

#[derive(Component)]
#[component(storage = "Table")]
pub struct VideoStream {
    async_rt: TokioAsyncRuntimeHandle,
    video_data_channel: Arc<RTCDataChannel>,
    frame_id: i32,
    encoder: VideoStreamEncoder,
    cuda_encoder: Option<NvEncEncoderWrapper>,
    rgb_to_yuv: RgbToYuvConverter,
    max_frame_time: u64,
}

impl VideoStream {
    #[span_fn]
    pub fn new(
        device_context: &DeviceContext,
        pipeline_manager: &PipelineManager,
        resolution: Resolution,
        encoder_work_queue: &EncoderWorkQueue,
        video_data_channel: Arc<RTCDataChannel>,
        async_rt: TokioAsyncRuntimeHandle,
    ) -> anyhow::Result<Self> {
        let encoder = VideoStreamEncoder::new(resolution)?;
        let rgb_to_yuv = RgbToYuvConverter::new(pipeline_manager, device_context, resolution);
        let max_frame_time: u64 = lgn_config::get_or("streamer.max_frame_time", 33_000u64)?;

        let encoder_cofig = EncoderConfig {
            hardware: Nvidia,
            gfx_config: device_context.clone(),
            work_queue: encoder_work_queue.clone(),
            width: resolution.width,
            height: resolution.height,
        };

        Ok(Self {
            async_rt,
            video_data_channel,
            frame_id: 0,
            encoder,
            cuda_encoder: NvEncEncoderWrapper::new(encoder_cofig),
            rgb_to_yuv,
            max_frame_time,
        })
    }

    #[span_fn]
    pub(crate) fn resize(
        &mut self,
        device_context: &DeviceContext,
        extents: RenderSurfaceExtents,
    ) -> anyhow::Result<()> {
        let resolution = Resolution::new(extents.width(), extents.height());
        if self.rgb_to_yuv.resize(device_context, resolution) {
            self.encoder = VideoStreamEncoder::new(resolution)?;
        }
        Ok(())
    }

    fn record_frame_id_metric(&self) {
        imetric!("Frame ID begin present", "frame_id", self.frame_id as u64);
    }

    #[span_fn]
    pub(crate) fn present(
        &mut self,
        render_context: &RenderContext<'_>,

        render_surface: &mut RenderSurface,
    ) {
        self.record_frame_id_metric();
        let now = tokio::time::Instant::now();

        let chunks = if let Some(encoder) = &self.cuda_encoder {
            encoder
                .submit_input(&EncoderWorkItem {
                    image: render_surface.export_texture().clone(),
                    semaphore: render_surface.encoder_sem().clone(),
                })
                .unwrap();
            let output = encoder.query_output().unwrap();
            self.encoder.encode_cuda(&output[..], self.frame_id)
        } else {
            self.rgb_to_yuv
                .convert(
                    render_context,
                    render_surface,
                    self.encoder.yuv_holder.yuv.as_mut_slice(),
                )
                .unwrap();
        };


            self.encoder.encode(self.frame_id)
        };

        let elapsed = now.elapsed().as_micros() as u64;
        record_frame_time_metric(elapsed);

        if elapsed >= self.max_frame_time {
            warn!(
                "stream: frame {:?} took {}ms",
                self.frame_id,
                elapsed / 1000
            );
        }

        let frame_id = self.frame_id;
        self.frame_id += 1;

        let video_data_channel = Arc::clone(&self.video_data_channel);
        self.async_rt.start_detached(async move {
            // TODO: Temporarily shuts the warning about the closed data channel.
            // We need to understand why the render surface is not removed on connection close
            if matches!(
                video_data_channel.ready_state(),
                RTCDataChannelState::Closing | RTCDataChannelState::Closed
            ) {
                return;
            }

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
        });
    }
}

impl Presenter for VideoStream {
    fn resize(&mut self, device_context: &DeviceContext, extents: RenderSurfaceExtents) {
        self.resize(device_context, extents).unwrap();
    }
    fn present(&mut self, render_context: &RenderContext<'_>, render_surface: &mut RenderSurface) {
        self.present(render_context, render_surface);
    }
}

pub struct YuvHolder {
    yuv: Vec<u8>,
    width: usize,
    height: usize,
}

impl YuvHolder {
    /// Allocates a new helper for the given format.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            yuv: vec![0u8; (3 * (width * height)) / 2],
            width,
            height,
        }
    }
}

struct VideoStreamEncoder {
    encoder: Encoder,
    yuv_holder: YuvHolder,
    resolution: Resolution,
    mp4: Mp4Stream,
    writer: Cursor<Vec<u8>>,
    write_index: bool,
}

impl VideoStreamEncoder {
    #[span_fn]
    fn new(resolution: Resolution) -> anyhow::Result<Self> {
        let config = encoder::EncoderConfig::new(resolution.width(), resolution.height())
            .constant_sps(true)
            .max_fps(60.0)
            .skip_frame(false)
            .bitrate_bps(30_000_000);

        let encoder = encoder::Encoder::with_config(config)?;

        let yuv_holder = YuvHolder::new(resolution.width() as usize, resolution.height() as usize);

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
            yuv_holder,
            resolution,
            mp4,
            writer,
            write_index: true,
        })
    }

    #[span_fn]
    fn encode_cuda(&mut self, mut data: &[u8], frame_id: i32) -> Vec<Bytes> {
        if self.write_index {
            let sps = next_nalu(&mut data);
            assert_eq!(sps[4], 0x67);
            let pps = next_nalu(&mut data);
            assert_eq!(pps[4], 0x68);
            self.mp4
                .write_index(
                    &MediaConfig::Avc(AvcConfig {
                        width: self.resolution.width().try_into().unwrap(),
                        height: self.resolution.height().try_into().unwrap(),
                        seq_param_set: sps[4..].into(),
                        pic_param_set: pps[4..].into(),
                    })
                    .into(),
                    &mut self.writer,
                )
                .unwrap();
            self.write_index = false;
        }
        while !data.is_empty() {
            let nalu = next_nalu(&mut data);
            let size = nalu.len() - 4;
            let mut vec = vec![];
            vec.extend_from_slice(nalu);
            vec[0] = (size >> 24) as u8;
            vec[1] = ((size >> 16) & 0xFF) as u8;
            vec[2] = ((size >> 8) & 0xFF) as u8;
            vec[3] = (size & 0xFF) as u8;
            assert!(nalu[4] == 0x65 || nalu[4] == 0x61);
            self.mp4
                .write_sample(nalu[4] == 0x65, &vec, &mut self.writer)
                .unwrap();
        }

        let chunks = split_frame_in_chunks(self.writer.get_ref(), frame_id);
        self.writer.get_mut().clear();
        self.writer.set_position(0);
        chunks
    }

    #[span_fn]
    fn encode(&mut self, frame_id: i32) -> Vec<Bytes> {
        let stream = self.encoder.encode(&self.yuv_holder).unwrap();
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
                            &MediaConfig::Avc(AvcConfig {
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
                        stream.frame_type == encoder::FrameType::Idr,
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

#[allow(unsafe_code)]
fn split_frame_in_chunks(data: &[u8], frame_id: i32) -> Vec<Bytes> {
    // let mut file = std::fs::File::options()
    //     .write(true)
    //     .create(true)
    //     .append(true)
    //     .open("d:\\gpu_test.h264")
    //     .unwrap();
    // file.write_all(data).unwrap();

    const HEADER_SIZE: usize = 12;
    const MAX_CHUNK_SIZE: usize = 65536;
    const CHUNK_SIZE: usize = MAX_CHUNK_SIZE - HEADER_SIZE;
    let mut chunks = vec![];
    let chunk_count = ((data.len() - 1) / CHUNK_SIZE) + 1;
    chunks.reserve(chunk_count);

    for (chunk_index, data) in data.chunks(CHUNK_SIZE).enumerate() {
        let mut chunk = bytes::BytesMut::with_capacity(data.len() + HEADER_SIZE);
        chunk.put_i32_le(frame_id);
        chunk.put_i32_le(chunk_count.try_into().unwrap());
        chunk.put_i32_le(chunk_index.try_into().unwrap());
        chunk.put_slice(data);
        chunks.push(chunk.into());
    }

    chunks
}

fn next_nalu<'a, 'b>(data: &'a mut &'b [u8]) -> &'b [u8] {
    if data.len() < 4 || data[0] != 0 || data[1] != 0 || data[2] != 0 || data[3] != 1 {
        return &[];
    }
    let mut i = 4;
    while data.len() - i > 4 {
        if data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 0 && data[i + 3] == 1 {
            let output = &data[0..i];
            *data = &data[i..];
            return output;
        }
        i += 1;
    }
    let output = *data;
    *data = &[];
    output
}

#[allow(clippy::cast_possible_wrap)]
impl YUVSource for YuvHolder {
    fn width(&self) -> i32 {
        self.width as i32
    }

    fn height(&self) -> i32 {
        self.height as i32
    }

    fn y(&self) -> &[u8] {
        &self.yuv[0..self.width * self.height]
    }

    fn u(&self) -> &[u8] {
        let base_u = self.width * self.height;
        &self.yuv[base_u..base_u + base_u / 4]
    }

    fn v(&self) -> &[u8] {
        let base_u = self.width * self.height;
        let base_v = base_u + base_u / 4;
        &self.yuv[base_v..]
    }

    fn y_stride(&self) -> i32 {
        self.width as i32
    }

    fn u_stride(&self) -> i32 {
        (self.width / 2) as i32
    }

    fn v_stride(&self) -> i32 {
        (self.width / 2) as i32
    }
}
