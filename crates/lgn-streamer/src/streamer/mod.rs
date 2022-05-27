use std::sync::Arc;

use bytes::Bytes;
use lgn_app::{AppExit, Events};
use lgn_async::TokioAsyncRuntime;
use lgn_codec_api::stream_encoder::StreamEncoder;
use lgn_ecs::prelude::*;
use lgn_graphics_renderer::{
    components::{RenderSurface, RenderSurfaceCreatedForWindow, RenderSurfaces},
    resources::PipelineManager,
    Renderer,
};
use lgn_input::{
    gamepad::GamepadEventRaw,
    keyboard::KeyboardInput,
    mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    touch::TouchInput,
};
use lgn_tracing::{error, info, trace, warn};
use lgn_window::{
    CursorMoved, Window, WindowCloseRequested, WindowCreated, WindowDescriptor, WindowId,
    WindowResized, Windows,
};
use serde_json::json;
use webrtc::{
    data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    peer_connection::RTCPeerConnection,
};

pub(crate) mod control_stream;
use control_stream::{ControlStream, ControlStreams};

mod events;
pub(crate) use events::*;

pub(crate) mod streamer_windows;
use streamer_windows::StreamerWindows;

mod video_stream;
use video_stream::VideoStream;

mod rgb2yuv;
use rgb2yuv::RgbToYuvConverter;

mod hdr2rgb;
use hdr2rgb::Hdr2Rgb;

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

// Stream-related events.
pub(crate) enum StreamEvent {
    ConnectionEstablished(
        Arc<RTCPeerConnection>,
        tokio::sync::oneshot::Sender<WindowId>,
    ),
    ConnectionClosed(WindowId, Arc<RTCPeerConnection>),
    VideoChannelOpened(WindowId, Arc<RTCDataChannel>),
    VideoChannelClosed(WindowId, Arc<RTCDataChannel>),
    VideoChannelMessageReceived(WindowId, Arc<RTCDataChannel>, DataChannelMessage),
    ControlChannelOpened(WindowId, Arc<RTCDataChannel>),
    ControlChannelClosed(WindowId, Arc<RTCDataChannel>),
    ControlChannelMessageReceived(WindowId, Arc<RTCDataChannel>, DataChannelMessage),
}

impl Streamer {
    pub(crate) fn new(stream_events_receiver: crossbeam::channel::Receiver<StreamEvent>) -> Self {
        Self {
            stream_events_receiver,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_stream_events(
    async_rt: Res<'_, TokioAsyncRuntime>,
    streamer: Res<'_, Streamer>,
    mut control_events: EventWriter<'_, '_, ControlEvent>,
    mut video_stream_events: EventWriter<'_, '_, VideoStreamEvent>,
    mut window_list: ResMut<'_, Windows>,
    mut streamer_windows: ResMut<'_, StreamerWindows>,
    mut control_streams: ResMut<'_, ControlStreams>,
    mut window_close_requested_events: ResMut<'_, Events<WindowCloseRequested>>,
) {
    for event in streamer.stream_events_receiver.try_iter() {
        match event {
            StreamEvent::ConnectionEstablished(_, sender) => {
                let window_id = WindowId::new();

                info!("Connection is now established for WindowId {}", window_id,);

                if let Err(e) = sender.send(window_id) {
                    warn!("Failed to send back window id ({}): {}", window_id, e);
                }
            }
            StreamEvent::ConnectionClosed(window_id, _) => {
                window_close_requested_events.send(WindowCloseRequested { id: window_id });
                streamer_windows.remove_mapping(&window_id);
                window_list.remove(&window_id);
                info!(
                    "Connection was closed for WindowId {}: closing window",
                    window_id,
                );
            }
            StreamEvent::VideoChannelOpened(window_id, data_channel) => {
                streamer_windows.add_mapping(window_id, data_channel.clone());
                info!(
                    "Video channel is now opened for WindowId {}: adding a video-stream mapping",
                    window_id,
                );
            }
            StreamEvent::VideoChannelClosed(window_id, _) => {
                streamer_windows.remove_mapping(&window_id);
                info!(
                    "Video channel is now closed for stream {}: removing video-stream mapping",
                    window_id,
                );
            }
            StreamEvent::VideoChannelMessageReceived(window_id, _data_channel, msg) => {
                match VideoStreamEvent::parse(window_id, &msg.data) {
                    Ok(event) => {
                        video_stream_events.send(event);
                    }
                    Err(e) => {
                        warn!("Ignoring unknown video data channel message: {}", e);
                    }
                }
            }
            StreamEvent::ControlChannelOpened(window_id, data_channel) => {
                let mut control_stream = ControlStream::new(data_channel);
                match control_stream.say_hello() {
                    Ok(future) => {
                        async_rt.start_detached(future);
                    }
                    Err(e) => {
                        error!("say_hello failed: {}", e);
                    }
                }
                control_streams.0.insert(window_id, control_stream);

                info!(
                    "Control channel is now opened for WindowId {}: adding a control-stream",
                    window_id,
                );
            }
            StreamEvent::ControlChannelClosed(window_id, _) => {
                control_streams.0.remove(&window_id);

                info!(
                    "Control channel is now closed for WindowId {}: removing control-stream",
                    window_id,
                );
            }
            StreamEvent::ControlChannelMessageReceived(stream_id, data_channel, msg) => {
                match ControlEvent::parse(stream_id, data_channel, &msg.data) {
                    Ok(event) => {
                        control_events.send(event);
                    }
                    Err(e) => {
                        warn!("Ignoring unknown video data channel message: {}", e);
                    }
                }
            }
        }
    }
}

pub(crate) fn handle_control_events(
    mut control_events: EventReader<'_, '_, ControlEvent>,
    mut render_surfaces: ResMut<'_, RenderSurfaces>,
) {
    for event in control_events.iter() {
        match &event.info {
            ControlEventInfo::Pause => {
                trace!("Received control Pause event, pausing stream");

                render_surfaces.for_each_mut(|render_surface| {
                    render_surface.pause();
                });
            }
            ControlEventInfo::Resume => {
                trace!("Received control Resume event, resuming stream");

                render_surfaces.for_each_mut(|render_surface| {
                    render_surface.resume();
                });
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_streams(
    mut video_stream_events: EventReader<'_, '_, VideoStreamEvent>,
    mut input_mouse_motion: EventWriter<'_, '_, MouseMotion>,
    mut input_mouse_button_input: EventWriter<'_, '_, MouseButtonInput>,
    mut input_mouse_wheel: EventWriter<'_, '_, MouseWheel>,
    mut input_touch_input: EventWriter<'_, '_, TouchInput>,
    mut input_keyboard_input: EventWriter<'_, '_, KeyboardInput>,
    mut input_cursor_moved: EventWriter<'_, '_, CursorMoved>,
    mut input_gamepad_events: EventWriter<'_, '_, GamepadEventRaw>,
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

                #[allow(clippy::cast_precision_loss)]
                let window_descriptor = WindowDescriptor {
                    width: *width as f32,
                    height: *height as f32,
                    ..WindowDescriptor::default()
                };

                window_list.add(Window::new(
                    event.window_id,
                    &window_descriptor,
                    *width,
                    *height,
                    1.0,
                    None,
                ));

                created_events.send(WindowCreated {
                    id: event.window_id,
                });
            }
            VideoStreamEventInfo::Resize { width, height } => {
                if let Some(window) = window_list.get_mut(event.window_id) {
                    window.update_actual_size_from_backend(*width, *height);

                    #[allow(clippy::cast_precision_loss)]
                    resize_events.send(WindowResized {
                        id: event.window_id,
                        width: *width as f32,
                        height: *height as f32,
                    });
                }
            }
            VideoStreamEventInfo::Speed { speed } => {
                error!("received unimplemented speed command {}", speed);
            }
            VideoStreamEventInfo::Input { input } => {
                trace!("received input: {:?}", input);

                match input {
                    Input::MouseButtonInput(mouse_button_input) => {
                        input_mouse_button_input.send(mouse_button_input.clone());
                    }
                    Input::MouseMotion(mouse_motion) => {
                        input_mouse_motion.send(MouseMotion {
                            delta: mouse_motion.delta,
                        });
                        input_cursor_moved.send(CursorMoved {
                            id: WindowId::primary(),
                            position: mouse_motion.current,
                        });
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
                    Input::GamepadConnection(gamepad_connection) => {
                        input_gamepad_events.send(gamepad_connection.into());
                    }
                    Input::GamepadDisconnection(gamepad_disconnection) => {
                        input_gamepad_events.send(gamepad_disconnection.into());
                    }
                    Input::GamepadButtonChange(gamepad_button_change) => {
                        input_gamepad_events.send(gamepad_button_change.into());
                    }
                    Input::GamepadAxisChange(gamepad_axis_change) => {
                        input_gamepad_events.send(gamepad_axis_change.into());
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
    mut render_surfaces: ResMut<'_, RenderSurfaces>,
) {
    if app_exit.iter().last().is_some() {
        render_surfaces.clear();
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn on_render_surface_created_for_window(
    mut event_render_surface_created: EventReader<'_, '_, RenderSurfaceCreatedForWindow>,
    wnd_list: Res<'_, Windows>,
    streamer_windows: Res<'_, StreamerWindows>,
    renderer: Res<'_, Renderer>,
    pipeline_manager: Res<'_, PipelineManager>,
    stream_encoder: Res<'_, StreamEncoder>,
    mut render_surfaces: ResMut<'_, RenderSurfaces>,
    async_rt: Res<'_, TokioAsyncRuntime>,
) {
    let device_context = renderer.device_context();

    for event in event_render_surface_created.iter() {
        let render_surface = render_surfaces.get_from_window_id_mut(event.window_id);
        let wnd = wnd_list.get(event.window_id).unwrap();

        let video_data_channel = streamer_windows
            .get_video_data_channel(event.window_id)
            .unwrap();

        let video_stream = VideoStream::new(
            device_context,
            &pipeline_manager,
            Resolution::new(wnd.physical_width(), wnd.physical_height()),
            &stream_encoder,
            video_data_channel.clone(),
            async_rt.handle(),
        )
        .unwrap();
        render_surface.register_presenter(|| video_stream);

        let _ = video_data_channel.send(&Bytes::from(json!({ "type": "initialized"}).to_string()));
    }
}
