//! `gRPC` plugin for Legion's ECS.
//!
//! Provides `gRPC` server support.

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
#![allow()]

use std::net::SocketAddr;

use legion_app::prelude::*;

use log::warn;

pub mod server;
pub mod service;

pub struct GRPCPluginSettings {
    grpc_server_addr: SocketAddr,
}

impl Default for GRPCPluginSettings {
    fn default() -> Self {
        Self {
            grpc_server_addr: "[::1]:50051".parse().unwrap(),
        }
    }
}

// Provides gRPC server capabilities to the engine.
pub struct GRPCPlugin {}

//impl GRPCServerRegistry {
//    async fn serve(self, addr: SocketAddr) -> Result<(), tonic::transport::Error> {
//        match tonic::transport::Server::builder()
//            .add_service(self.service)
//            .serve(addr)
//            .await
//        {
//            Ok(_) => Ok(()),
//            Err(e) => {
//                warn!("gRPC server stopped and no longer listening ({})", e);
//
//                Err(e)
//            }
//        }
//    }
//}

impl Plugin for GRPCPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GRPCPluginSettings>();

        //app.init_resource::<GRPCServerRegistry>();
        //app.add_startup_system(GRPCPlugin::start_grpc_server);
    }
}

//impl GRPCPlugin {
//    fn start_grpc_server(
//        mut server: ResMut<'_, GRPCServerRegistry>,
//        settings: Res<'_, GRPCPluginSettings>,
//        rt: ResMut<'_, legion_async::TokioAsyncRuntime>,
//    ) {
//        rt.start_detached(server.serve(settings.grpc_server_addr));
//    }
//}
