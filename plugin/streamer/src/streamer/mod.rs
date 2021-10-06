use legion_async::TokioAsyncRuntime;
use legion_ecs::prelude::*;

use std::{fmt::Display, sync::Arc};
use webrtc::{
    data::data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    peer::peer_connection::RTCPeerConnection,
};

use log::{info, warn};

mod control_stream;
mod events;
mod video_stream;

use control_stream::ControlStream;
pub(crate) use events::*;
use video_stream::VideoStream;

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

    pub(crate) fn handle_stream_events(
        streamer: ResMut<'_, Self>,
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
                    commands
                        .entity(stream_id.entity)
                        .insert(ControlStream::new(data_channel));

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
        async_rt: ResMut<'_, TokioAsyncRuntime>,
        mut query: Query<'_, '_, (Option<&mut ControlStream>, Option<&mut VideoStream>)>,
        mut video_stream_events: EventReader<'_, '_, VideoStreamEvent>,
    ) {
        for event in video_stream_events.iter() {
            if let Ok(mut video_stream) =
                query.get_component_mut::<VideoStream>(event.stream_id.entity)
            {
                match event.info {
                    VideoStreamEventInfo::Hue { hue } => {
                        video_stream.set_hue(hue);
                    }
                    VideoStreamEventInfo::Resize { width, height } => {
                        video_stream.resize(width, height);
                    }
                }
            }
        }

        for (_, mut video_stream) in query.iter_mut() {
            if let Some(video_stream) = &mut video_stream {
                async_rt.start_detached(video_stream.render());
            }
        }
    }
}
