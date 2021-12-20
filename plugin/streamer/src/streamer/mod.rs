use std::{fmt::Display, sync::Arc};

use lgn_ecs::prelude::*;
use lgn_input::{
    keyboard::KeyboardInput,
    mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    touch::TouchInput,
};
use lgn_presenter::offscreen_helper::Resolution;
use lgn_renderer::{
    components::{RenderSurface, RenderSurfaceExtents},
    RenderTaskPool, Renderer,
};
use log::{error, info, warn};
use webrtc::{
    data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    peer_connection::RTCPeerConnection,
};

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
}

pub(crate) fn handle_stream_events(
    task_pool: Res<'_, RenderTaskPool>,
    streamer: Res<'_, Streamer>,
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
                let mut render_surface = RenderSurface::new(
                    &renderer,
                    RenderSurfaceExtents::new(resolution.width(), resolution.height()),
                );
                render_surface.register_presenter(|| {
                    VideoStream::new(&renderer, resolution, data_channel).unwrap()
                });
                commands.entity(stream_id.entity).insert(render_surface);

                info!(
                    "Video channel is now opened for stream {}: adding a video-stream component",
                    stream_id,
                );
            }
            StreamEvent::VideoChannelClosed(stream_id, _) => {
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
                        task_pool.spawn(future).detach();
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_streams(
    renderer: Res<'_, Renderer>,
    mut query: Query<'_, '_, &mut RenderSurface>,
    mut video_stream_events: EventReader<'_, '_, VideoStreamEvent>,
    mut input_mouse_motion: EventWriter<'_, '_, MouseMotion>,
    mut input_mouse_button_input: EventWriter<'_, '_, MouseButtonInput>,
    mut input_mouse_wheel: EventWriter<'_, '_, MouseWheel>,
    mut input_touch_input: EventWriter<'_, '_, TouchInput>,
    mut input_keyboard_input: EventWriter<'_, '_, KeyboardInput>,
) {
    for event in video_stream_events.iter() {
        match query.get_mut(event.stream_id.entity) {
            Ok(mut render_surface) => {
                let render_pass = render_surface.test_renderpass();

                match &event.info {
                    VideoStreamEventInfo::Color { id, color } => {
                        log::info!("received color command id={}", id);
                        render_pass.write().set_color(color.0);
                    }
                    VideoStreamEventInfo::Resize { width, height } => {
                        let resolution = Resolution::new(*width, *height);
                        render_surface.resize(
                            &renderer,
                            RenderSurfaceExtents::new(resolution.width(), resolution.height()),
                        );
                    }
                    VideoStreamEventInfo::Speed { id, speed } => {
                        log::info!("received speed command id={}", id);
                        render_pass.write().set_speed(*speed);
                    }
                    VideoStreamEventInfo::Input { input } => {
                        log::info!("received input: {:?}", input);

                        match input {
                            Input::MouseButtonInput(mouse_button_input) => {
                                input_mouse_button_input.send(mouse_button_input.clone());
                            }
                            Input::MouseMotion(mouse_motion) => {
                                input_mouse_motion.send(mouse_motion.clone());
                            }
                            Input::MouseWheel(mouse_wheel) => {
                                input_mouse_wheel.send(mouse_wheel.clone());
                            }
                            Input::TouchInput(touch_input) => {
                                // A bit unsure why, but unlike other events [`legion_input::touch:TouchInput`]
                                // derives Copy (_and_ `Clone`).
                                input_touch_input.send(*touch_input);
                            }
                            Input::KeyboardInput(keyboard_input) => {
                                input_keyboard_input.send(keyboard_input.clone());
                            }
                        }
                    }
                }
            }
            Err(query_err) => {
                // TODO
                // Most likely: "The given entity does not have the requested component"
                // i.e. the entity associated with the stream-id does not have a RenderSurface
                eprintln!("{}", query_err);
            }
        }
    }
}
