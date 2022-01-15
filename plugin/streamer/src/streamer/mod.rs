use std::{fmt::Display, sync::Arc};

use bytes::Bytes;
use lgn_app::{AppExit, Events};
use lgn_async::TokioAsyncRuntime;
use lgn_ecs::prelude::*;
use lgn_input::{
    keyboard::KeyboardInput,
    mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    touch::TouchInput,
};
use lgn_renderer::{
    components::{RenderSurface, RenderSurfaceCreatedForWindow},
    Renderer,
};
use lgn_tracing::{error, info, trace, warn};
use lgn_window::{WindowCreated, WindowResized, Windows};
use webrtc::{
    data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    peer_connection::RTCPeerConnection,
};

mod control_stream;
use control_stream::ControlStream;

mod events;
pub(crate) use events::*;

pub(crate) mod streamer_windows;
use streamer_windows::StreamerWindows;

mod video_stream;
use video_stream::VideoStream;

mod rgb2yuv;
use rgb2yuv::RgbToYuvConverter;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Resolution {
    width: u32,
    height: u32,
}

impl Resolution {
    pub fn new(mut width: u32, mut height: u32) -> Self {
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

    pub fn width(self) -> u32 {
        self.width
    }

    pub fn height(self) -> u32 {
        self.height
    }
}

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
    async_rt: Res<'_, TokioAsyncRuntime>,
    streamer: Res<'_, Streamer>,
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
                //commands.entity(stream_id.entity).despawn();

                info!(
                    "Connection was closed for stream {}: despawning entity",
                    stream_id,
                );
            }
            StreamEvent::VideoChannelOpened(stream_id, _data_channel) => {
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
            StreamEvent::VideoChannelMessageReceived(stream_id, data_channel, msg) => {
                match VideoStreamEvent::parse(stream_id, data_channel, &msg.data) {
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_streams(
    mut query: Query<'_, '_, &mut RenderSurface>,
    mut video_stream_events: EventReader<'_, '_, VideoStreamEvent>,
    mut input_mouse_motion: EventWriter<'_, '_, MouseMotion>,
    mut input_mouse_button_input: EventWriter<'_, '_, MouseButtonInput>,
    mut input_mouse_wheel: EventWriter<'_, '_, MouseWheel>,
    mut input_touch_input: EventWriter<'_, '_, TouchInput>,
    mut input_keyboard_input: EventWriter<'_, '_, KeyboardInput>,
    mut streamer_windows: ResMut<'_, StreamerWindows>,
    mut window_list: ResMut<'_, Windows>,
    mut created_events: ResMut<'_, Events<WindowCreated>>,
    mut resize_events: ResMut<'_, Events<WindowResized>>,
) {
    for event in video_stream_events.iter() {
        match &event.info {
            VideoStreamEventInfo::Initialize {
                color: _,
                width,
                height,
            } => {
                trace!("received initialize command");

                window_list.add(streamer_windows.create_window(
                    event.stream_id,
                    Resolution::new(*width, *height),
                    Arc::clone(&event.video_data_channel),
                    &mut created_events,
                ));
            }
            VideoStreamEventInfo::Resize { width, height } => {
                if let Some(window_id) = streamer_windows.get_window_id(event.stream_id) {
                    let window = window_list.get_mut(window_id).unwrap();

                    window.update_actual_size_from_backend(*width, *height);

                    #[allow(clippy::cast_precision_loss)]
                    resize_events.send(WindowResized {
                        id: window_id,
                        width: *width as f32,
                        height: *height as f32,
                    });
                }
            }
            VideoStreamEventInfo::Speed { speed } => {
                trace!("received speed command {}", speed);

                match query.get_mut(event.stream_id.entity) {
                    Ok(render_surface) => {
                        let render_pass = render_surface.test_renderpass();

                        render_pass.write().set_speed(*speed);
                    }
                    Err(query_err) => {
                        // TODO
                        // Most likely: "The given entity does not have the requested component"
                        // i.e. the entity associated with the stream-id does not have a RenderSurface
                        error!("{}", query_err);
                    }
                }
            }
            VideoStreamEventInfo::Input { input } => {
                trace!("received input: {:?}", input);

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
                        // A bit unsure why, but unlike other events
                        // [`legion_input::touch:TouchInput`]
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
}

// Cleaning up render surfaces otherwise we crash on Ctrl+C
pub(crate) fn on_app_exit(
    mut commands: Commands<'_, '_>,
    mut app_exit: EventReader<'_, '_, AppExit>,
    query_render_surface: Query<'_, '_, (Entity, &RenderSurface)>,
) {
    if app_exit.iter().last().is_some() {
        for (entity, _) in query_render_surface.iter() {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn on_render_surface_created_for_window(
    mut event_render_surface_created: EventReader<'_, '_, RenderSurfaceCreatedForWindow>,
    wnd_list: Res<'_, Windows>,
    streamer_windows: Res<'_, StreamerWindows>,
    renderer: Res<'_, Renderer>,
    mut render_surfaces: Query<'_, '_, &mut RenderSurface>,
    async_rt: Res<'_, TokioAsyncRuntime>,
) {
    for event in event_render_surface_created.iter() {
        let render_surface = render_surfaces
            .iter_mut()
            .find(|x| x.id() == event.render_surface_id);
        if let Some(mut render_surface) = render_surface {
            let wnd = wnd_list.get(event.window_id).unwrap();

            let video_data_channel = streamer_windows
                .get_video_data_channel(event.window_id)
                .unwrap();

            let video_stream = VideoStream::new(
                &renderer,
                Resolution::new(wnd.physical_width(), wnd.physical_height()),
                video_data_channel.clone(),
                async_rt.handle(),
            )
            .unwrap();
            render_surface.register_presenter(|| video_stream);

            let _ = video_data_channel.send(&Bytes::from(r#"{"type": "initialized"}"#));
        }
    }
}
