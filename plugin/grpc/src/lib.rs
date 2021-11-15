//! `gRPC` plugin for Legion's ECS.
//!
//! Provides `gRPC` server support to the engine, compatible with the `tonic` crate.

// BEGIN - Legion Labs lints v0.6
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
    clippy::if_not_else,
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
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow()]

use std::net::SocketAddr;

use legion_app::prelude::*;
use legion_ecs::prelude::*;
use legion_online::grpc::multiplexer_service::{
    MultiplexableService, MultiplexerService, MultiplexerServiceBuilder,
};
use log::{info, warn};
use tonic::transport::NamedService;

pub struct GRPCPluginSettings {
    pub grpc_server_addr: SocketAddr,
    multiplexer_service_builder: MultiplexerServiceBuilder,
}

impl GRPCPluginSettings {
    pub fn new(grpc_server_addr: SocketAddr) -> Self {
        Self {
            grpc_server_addr,
            multiplexer_service_builder: MultiplexerService::builder(),
        }
    }

    pub fn register_service<S>(&mut self, s: S) -> &mut Self
    where
        S: MultiplexableService + NamedService + Send + Sync + 'static,
    {
        self.multiplexer_service_builder.add_service(s);

        self
    }
}

impl Default for GRPCPluginSettings {
    fn default() -> Self {
        Self::new("[::1]:50051".parse().unwrap())
    }
}

// Provides gRPC server capabilities to the engine.
#[derive(Default)]
pub struct GRPCPlugin;

impl Plugin for GRPCPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GRPCPluginSettings>();
        app.add_startup_system(Self::start_grpc_server);
    }
}

impl GRPCPlugin {
    #[allow(clippy::needless_pass_by_value)]
    fn start_grpc_server(
        settings: Res<'_, GRPCPluginSettings>,
        rt: ResMut<'_, legion_async::TokioAsyncRuntime>,
    ) {
        if let Some(service) = settings.multiplexer_service_builder.build() {
            let server = tonic::transport::Server::builder().add_service(service);
            let addr = settings.grpc_server_addr;

            rt.start_detached(async move {
                info!("starting gRPC server on {}", addr);

                match server.serve(addr).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        warn!("gRPC server stopped and no longer listening ({})", e);

                        Err(e)
                    }
                }
            });
        } else {
            warn!("not starting gRPC server as no service was registered");
        }
    }
}
