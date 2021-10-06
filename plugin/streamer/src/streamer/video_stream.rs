use legion_ecs::prelude::*;

use log::{debug, info, warn};
use std::sync::Arc;

use webrtc::data::data_channel::RTCDataChannel;

use legion_codec_api::{
    backends::openh264::encoder::{self, Encoder, FrameType},
    formats::{self, RBGYUVConverter},
};
use legion_mp4::Mp4Stream;

#[derive(Component)]
#[component(storage = "Table")]
pub struct VideoStream {
    video_data_channel: Arc<RTCDataChannel>,
    src: &'static [u8],
    encoder: Encoder,
    converter: RBGYUVConverter,
    mp4: Mp4Stream,
    track_id: i32,
    frame_id: i32,
    hue: f32,
}

impl VideoStream {
    pub fn new(video_data_channel: Arc<RTCDataChannel>) -> anyhow::Result<Self> {
        // Sample code to stream a video.
        //let src = &include_bytes!("../../../server/editor-srv/assets/lenna_512x512.rgb")[..];
        let src = &include_bytes!("../../../../server/editor-srv/assets/pencils_1024x768.rgb")[..];

        struct Resolution {
            width: u32,
            height: u32,
        }

        let resolution = Resolution {
            width: 1024,
            height: 768,
        };

        let config = encoder::EncoderConfig::new(resolution.width, resolution.height)
            .constant_sps(true)
            .max_fps(60.0)
            .skip_frame(false)
            .bitrate_bps(8_000_000);

        let encoder = encoder::Encoder::with_config(config)?;
        let converter =
            formats::RBGYUVConverter::new(resolution.width as usize, resolution.height as usize);

        let mut mp4 = Mp4Stream::new(60);
        let track_id = mp4
            .add_track(resolution.width as i32, resolution.height as i32)
            .unwrap();

        Ok(Self {
            video_data_channel,
            src,
            encoder,
            converter,
            mp4,
            track_id,
            frame_id: 0,
            hue: 1.0,
        })
    }

    pub(crate) fn set_hue(&mut self, hue: f32) {
        self.hue = hue;
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        info!("Resizing stream to {}x{}", width, height);
    }

    pub(crate) fn render(&mut self) -> impl std::future::Future<Output = ()> + 'static {
        let now = tokio::time::Instant::now();

        let rgb_modulation = hue2rgb_modulation(self.hue);

        self.converter.convert_rgb(self.src, rgb_modulation);
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

        let max_chunk_size = 65536;
        let mut chunks = vec![];
        chunks.reserve(((data.len() - 1) / max_chunk_size) + 1);

        for data in data.chunks(max_chunk_size) {
            chunks.push(bytes::Bytes::copy_from_slice(data));
        }

        self.mp4.clean();

        let elapsed = now.elapsed().as_micros() as u64;
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

fn hue2rgb_modulation(hue: f32) -> (f32, f32, f32) {
    let rgb = hsl::HSL {
        h: (hue * 360.0) as f64,
        s: 1_f64,
        l: 0.5_f64,
    }
    .to_rgb();

    (
        rgb.0 as f32 / 256.0,
        rgb.1 as f32 / 256.0,
        rgb.2 as f32 / 256.0,
    )
}
