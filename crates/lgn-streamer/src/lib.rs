//! Crate doc

// crate-specific lint exceptions:
#![allow(clippy::let_underscore_drop, clippy::needless_pass_by_value)]
#![warn(missing_docs)]

use lgn_app::prelude::*;
use lgn_core::Time;

mod cgen {
    include!(concat!(env!("OUT_DIR"), "/rust/mod.rs"));
}

mod tmp_shader_data;

#[allow(unused_imports)]
use cgen::*;
use lgn_ecs::prelude::{Res, ResMut};
use lgn_graphics_cgen_runtime::CGenRegistryList;
use lgn_renderer::{resources::ShaderManager, Renderer};
use tmp_shader_data::patch_cgen_registry;

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
            .add_system(streamer::on_render_surface_created_for_window)
            .add_startup_system(init_cgen);

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

fn init_cgen(
    renderer: Res<'_, Renderer>,
    mut shader_manager: ResMut<'_, ShaderManager>,
    mut cgen_registries: ResMut<'_, CGenRegistryList>,
) {
    let mut cgen_registry = cgen::initialize(renderer.device_context());
    patch_cgen_registry(&mut cgen_registry);
    shader_manager.register_cgen_registry(&cgen_registry);
    cgen_registries.push(cgen_registry);
}
