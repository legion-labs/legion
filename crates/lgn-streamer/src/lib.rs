//! Crate doc

// crate-specific lint exceptions:
#![allow(
    clippy::let_underscore_drop,
    clippy::needless_pass_by_value,
    clippy::too_many_lines
)]
#![warn(missing_docs)]

use lgn_app::prelude::*;
use lgn_core::Time;

mod grpc;
mod streamer;
mod webrtc;

/// Provides streaming capabilities to the engine.
#[derive(Default)]
pub struct StreamerPlugin;

impl Plugin for StreamerPlugin {
    fn build(&self, app: &mut App) {
        // This channel is used a communication mechanism between the async server
        // threads and the game-loop.
        let (stream_events_sender, stream_events_receiver) = crossbeam::channel::unbounded();

        // The streamer is the game-loop representative of the whole streaming system.
        let streamer = streamer::Streamer::new(stream_events_receiver);

        app.insert_resource(streamer)
            .init_resource::<Time>()
            .init_resource::<streamer::streamer_windows::StreamerWindows>()
            .add_event::<streamer::VideoStreamEvent>()
            .add_system(streamer::handle_stream_events)
            .add_system(streamer::update_streams)
            .add_system(streamer::on_app_exit)
            .add_system(streamer::on_render_surface_created_for_window);

        let webrtc_server =
            webrtc::WebRTCServer::new().expect("failed to instanciate a WebRTC server");
        let grpc_server = grpc::GRPCServer::new(webrtc_server, stream_events_sender);

        app.world
            .get_resource_mut::<lgn_grpc::GRPCPluginSettings>()
            .expect("the streamer plugin requires the gRPC plugin")
            .into_inner()
            .register_service(grpc_server.service());
    }
}
