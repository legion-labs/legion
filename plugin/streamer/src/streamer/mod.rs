use legion_async::TokioAsyncRuntime;
use legion_core::Time;
use legion_ecs::prelude::*;
use legion_renderer::{components::RenderSurface, Renderer};

use std::{fmt::Display, sync::Arc};
use webrtc::{
    data::data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    peer::peer_connection::RTCPeerConnection,
};

use log::{error, info, warn};

mod control_stream;
mod events;
mod video_stream;

use control_stream::ControlStream;
pub(crate) use events::*;
use video_stream::VideoStream;

use crate::streamer::video_stream::Resolution;

// Streamer provides interaction with the async network components (gRPC &
// WebRTC) from the synchronous game-loop.
pub(crate) struct Streamer {
    stream_events_receiver: crossbeam::channel::Receiver<StreamEvent>,
}

// StreamID represents a stream unique identifier.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct StreamID {
    entity: Entity,
}

// Stream-related events.
pub(crate) enum StreamEvent {
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
    pub(crate) fn new(stream_events_receiver: crossbeam::channel::Receiver<StreamEvent>) -> Self {
        Self {
            stream_events_receiver,
        }
    }
}

pub(crate) fn handle_stream_events(
    async_rt: ResMut<'_, TokioAsyncRuntime>,
    streamer: ResMut<'_, Streamer>,
    renderer: Res<'_, Renderer>,
    mut commands: Commands<'_, '_>,
    mut video_stream_events: EventWriter<'_, '_, VideoStreamEvent>,
) {
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
                let resolution = Resolution::new(1024, 768);
                commands
                    .entity(stream_id.entity)
                    .insert(RenderSurface::new(
                        &renderer,
                        resolution.width(),
                        resolution.height(),
                    ))
                    .insert(VideoStream::new(&renderer, resolution, data_channel).unwrap());

                info!(
                    "Video channel is now opened for stream {}: adding a video-stream component",
                    stream_id,
                );
            }
            StreamEvent::VideoChannelClosed(stream_id, _) => {
                commands.entity(stream_id.entity).remove::<VideoStream>();
                commands.entity(stream_id.entity).remove::<RenderSurface>();

                info!(
                    "Video channel is now closed for stream {}: removing video-stream component",
                    stream_id,
                );
            }
            StreamEvent::VideoChannelMessageReceived(stream_id, _, msg) => {
                match VideoStreamEvent::parse(stream_id, &msg.data) {
                    Ok(event) => {
                        video_stream_events.send(event);
                    }
                    Err(e) => {
                        warn!("Ignoring unknown video data channel message: {}", e);
                    }
                }
            }
            StreamEvent::ControlChannelOpened(stream_id, data_channel) => {
                let mut control_stream = ControlStream::new(data_channel);
                match control_stream.say_hello() {
                    Ok(future) => {
                        async_rt.start_detached(future);
                    }
                    Err(e) => {
                        error!("say_hello failed: {}", e);
                    }
                }
                commands.entity(stream_id.entity).insert(control_stream);

                info!(
                    "Control channel is now opened for stream {}: adding a control-stream component",
                    stream_id,
                );
            }
            StreamEvent::ControlChannelClosed(stream_id, _) => {
                commands.entity(stream_id.entity).remove::<ControlStream>();

                info!(
                    "Control channel is now closed for stream {}: removing control-stream component",
                    stream_id,
                );
            }
            StreamEvent::ControlChannelMessageReceived(_, _, _) => {
                //commands
                //    .entity(stream_id.entity)
                //    .get_components(|stream: &mut ControlStream| {});

                //control_stream.parse_and_append(msg);
            }
        }
    }
}

pub(crate) fn update_streams(
    renderer: Res<'_, Renderer>,
    mut query: Query<'_, '_, (&mut VideoStream, &mut RenderSurface)>,
    mut video_stream_events: EventReader<'_, '_, VideoStreamEvent>,
) {
    for event in video_stream_events.iter() {
        let (mut video_stream, mut render_surface) = query.get_mut(event.stream_id.entity).unwrap();

        match &event.info {
            VideoStreamEventInfo::Color { id, color } => {
                log::info!("received color command id={}", id);
                render_surface.test_renderpass.color = color.0;
            }
            VideoStreamEventInfo::Resize { width, height } => {
                let resolution = Resolution::new(*width, *height);
                render_surface.resize(&renderer, resolution.width(), resolution.height());
                video_stream.resize(&renderer, resolution).unwrap();
            }
            VideoStreamEventInfo::Speed { id, speed } => {
                log::info!("received speed command id={}", id);
                render_surface.test_renderpass.speed = *speed;
            }
        }
    }
}

pub(crate) fn render_streams(
    async_rt: ResMut<'_, TokioAsyncRuntime>,
    renderer: Res<'_, Renderer>,
    mut query: Query<'_, '_, (&mut VideoStream, &mut RenderSurface)>,
    mut time: ResMut<'_, Time>,
) {
    time.update();

    let graphics_queue = renderer.graphics_queue();
    let wait_sem = renderer.frame_signal_semaphore();

    for (mut video_stream, render_surface) in query.iter_mut() {
        async_rt.start_detached(video_stream.render(
            graphics_queue,
            wait_sem,
            render_surface.into_inner(),
        ));
    }
}
