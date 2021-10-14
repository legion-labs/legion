//! Crate doc

// BEGIN - Legion Labs lints v0.5
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs standard lints v0.5
// crate-specific exceptions:
#![allow(clippy::let_underscore_drop, clippy::needless_pass_by_value)]
#![warn(missing_docs)]

use legion_app::prelude::*;
use legion_core::Time;

mod grpc;
mod streamer;
mod webrtc;

/// Provides streaming capabilities to the engine.
pub struct StreamerPlugin {}

impl Plugin for StreamerPlugin {
    fn build(&self, app: &mut App) {
        // This channel is used a communication mechanism between the async server threads and the game-loop.
        let (stream_events_sender, stream_events_receiver) = crossbeam::channel::unbounded();

        // The streamer is the game-loop representative of the whole streaming system.
        let streamer = streamer::Streamer::new(stream_events_receiver);

        let time = Time::default();

        app.insert_resource(streamer)
            .insert_resource(time)
            .add_event::<streamer::VideoStreamEvent>()
            .add_system(streamer::Streamer::handle_stream_events)
            .add_system(streamer::Streamer::update_streams);

        let webrtc_server =
            webrtc::WebRTCServer::new().expect("failed to instanciate a WebRTC server");
        let grpc_server = grpc::GRPCServer::new(webrtc_server, stream_events_sender);

        app.world
            .get_resource_mut::<legion_grpc::GRPCPluginSettings>()
            .expect("the streamer plugin requires the gRPC plugin")
            .into_inner()
            .register_service(grpc_server.service());
    }
}
