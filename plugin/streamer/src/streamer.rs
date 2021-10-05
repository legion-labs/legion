use legion_async::TokioAsyncRuntime;
use legion_codec_api::{
    backends::openh264::encoder::{self, Encoder, FrameType},
    formats::{self, RBGYUVConverter},
};
use legion_ecs::prelude::*;
use legion_mp4::Mp4Stream;

use std::{fmt::Display, sync::Arc};
use webrtc::{
    data::data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    peer::peer_connection::RTCPeerConnection,
};

use log::{debug, info, warn};

// Streamer provides interaction with the async network components (gRPC &
// WebRTC) from the synchronous game-loop.
pub struct Streamer {
    stream_events_receiver: crossbeam::channel::Receiver<StreamEvent>,
}

// StreamID represents a stream unique identifier.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StreamID {
    entity: Entity,
}

// Stream-related events.
pub enum StreamEvent {
    ConnectionEstablished(
        Arc<RTCPeerConnection>,
        tokio::sync::oneshot::Sender<StreamID>,
    ),
    ConnectionClosed(StreamID, Arc<RTCPeerConnection>),
    VideoChannelOpened(StreamID, Arc<RTCDataChannel>),
    VideoChannelClosed(StreamID, Arc<RTCDataChannel>),
    VideoChannelMessageReceived(StreamID, Arc<RTCDataChannel>, DataChannelMessage),
    ControlChannelOpened(StreamID, Arc<RTCDataChannel>),
    ControlChannelClosed(StreamID, Arc<RTCDataChannel>),
    ControlChannelMessageReceived(StreamID, Arc<RTCDataChannel>, DataChannelMessage),
}

impl StreamID {
    fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

impl Display for StreamID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.entity)
    }
}

impl Streamer {
    pub fn new(stream_events_receiver: crossbeam::channel::Receiver<StreamEvent>) -> Self {
        Self {
            stream_events_receiver,
        }
    }

    pub fn handle_stream_events(streamer: ResMut<'_, Self>, mut commands: Commands<'_, '_>) {
        for event in streamer.stream_events_receiver.try_iter() {
            match event {
                StreamEvent::ConnectionEstablished(_, sender) => {
                    let stream_id = StreamID::new(commands.spawn().id());

                    info!(
                        "Connection is now established for stream {}: spawning entity",
                        stream_id,
                    );

                    if let Err(e) = sender.send(stream_id) {
                        warn!("Failed to send back stream id ({}): despawning entity to avoid it being orphaned: {}", stream_id, e);
                        commands.entity(stream_id.entity).despawn();
                    }
                }
                StreamEvent::ConnectionClosed(stream_id, _) => {
                    commands.entity(stream_id.entity).despawn();

                    info!(
                        "Connection was closed for stream {}: despawning entity",
                        stream_id,
                    );
                }
                StreamEvent::VideoChannelOpened(stream_id, data_channel) => {
                    commands
                        .entity(stream_id.entity)
                        .insert(VideoStream::new(data_channel).unwrap());

                    info!(
                        "Video channel is now opened for stream {}: adding a video-stream component",
                        stream_id,
                    );
                }
                StreamEvent::VideoChannelClosed(stream_id, _) => {
                    commands.entity(stream_id.entity).remove::<VideoStream>();

                    info!(
                        "Video channel is now closed for stream {}: removing video-stream component",
                        stream_id,
                    );
                }
                _ => {}
            }
        }
    }

    pub fn handle_video_streams(
        async_rt: ResMut<'_, TokioAsyncRuntime>,
        mut query: Query<'_, '_, &mut VideoStream>,
    ) {
        for mut stream in query.iter_mut() {
            async_rt.start_detached(stream.render_frame());
        }
    }
}

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
}

impl VideoStream {
    pub fn new(video_data_channel: Arc<RTCDataChannel>) -> anyhow::Result<Self> {
        // Sample code to stream a video.
        //let src = &include_bytes!("../../../server/editor-srv/assets/lenna_512x512.rgb")[..];
        let src = &include_bytes!("../../../server/editor-srv/assets/pencils_1024x768.rgb")[..];

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
        })
    }

    fn render_frame(&mut self) -> impl std::future::Future<Output = ()> + 'static {
        let now = tokio::time::Instant::now();

        let mut rgb_modulation = (1.0, 1.0, 1.0);
        let mut increments = (0.01, 0.02, 0.04);

        modulate(&mut rgb_modulation.0, &mut increments.0);
        modulate(&mut rgb_modulation.1, &mut increments.1);
        modulate(&mut rgb_modulation.2, &mut increments.2);

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

fn modulate(input: &mut f32, increment: &mut f32) {
    *input += *increment;
    if *input > 1.0 {
        *input = 1.0;
        *increment *= -1.0;
    } else if *input < 0.0 {
        *input = 0.0;
        *increment *= -1.0;
    }
}
