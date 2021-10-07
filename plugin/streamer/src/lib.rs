//! Crate doc

// BEGIN - Legion Labs lints v0.4
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::from_iter_instead_of_collect,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_digit_groups,
    clippy::large_stack_arrays,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::missing_enforced_import_renames,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::single_match_else,
    clippy::string_add_assign,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::trait_duplication_in_bounds,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::useless_transmute,
    clippy::use_self,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// END - Legion Labs standard lints v0.4
// crate-specific exceptions:
#![allow(clippy::needless_pass_by_value)]
#![warn(missing_docs)]

use std::net::SocketAddr;

use legion_app::prelude::*;
use legion_core::Time;

mod grpc;
mod streamer;
mod webrtc;

/// Configuration for the `StreamerPlugin`.
pub struct StreamerPluginSettings {
    /// The listening address of the `gRPC` server.
    pub grpc_server_addr: SocketAddr,
}

impl Default for StreamerPluginSettings {
    fn default() -> Self {
        Self {
            grpc_server_addr: "[::1]:50051".parse().unwrap(),
        }
    }
}

/// Provides streaming capabilities to the engine.
pub struct StreamerPlugin {}

impl Plugin for StreamerPlugin {
    fn build(&self, app: &mut App) {
        let settings = app
            .world
            .remove_resource::<StreamerPluginSettings>()
            .map_or_else(StreamerPluginSettings::default, |x| x);

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

        // Let's limit our usage of the Async runtime, as this keeps a mutable
        // reference on the world.
        {
            let async_rt = app
                .world
                .get_resource_mut::<legion_async::TokioAsyncRuntime>()
                .expect("the streamer plugin requires the async plugin")
                .into_inner();

            async_rt.start_detached(grpc_server.listen_and_serve(settings.grpc_server_addr));
        }
    }
}
