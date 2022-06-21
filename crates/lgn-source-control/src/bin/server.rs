//! Legion Source Control Server
//!
//! TODO: write documentation.

// crate-specific lint exceptions:
//#![allow()]

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use clap::Parser;
use lgn_online::server::RouterExt;
use lgn_source_control::{api::source_control::server, Server};
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::{info, LevelFilter};
use serde::Deserialize;

#[derive(Parser, Debug)]
#[clap(name = "Legion Labs source-control server")]
#[clap(about = "Source-control server.", version, author)]
struct Args {
    #[clap(name = "debug", short, long, help = "Enable debug logging")]
    debug: bool,

    /// The address to listen on.
    #[clap(long, default_value = "0.0.0.0:5000")]
    listen_endpoint: SocketAddr,

    #[clap(
        long,
        default_value = "",
        help = "The list of origins that are allowed to make requests, for CORS"
    )]
    origins: Vec<http::HeaderValue>,
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    database: lgn_source_control::SqlConfig,
}

#[allow(clippy::semicolon_if_nothing_returned)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let _telemetry_guard = if args.debug {
        TelemetryGuardBuilder::default()
            .with_local_sink_max_level(LevelFilter::Debug)
            .build()
    } else {
        TelemetryGuardBuilder::default().build()
    };

    let config: Config = lgn_config::get("source_control.server")?
        .ok_or_else(|| anyhow::anyhow!("no configuration was found for `source_control.server`"))?;

    let repository_index = config.database.instantiate().await?;
    let server = Arc::new(Server::new(repository_index));
    let router = Router::new();
    let router = server::register_routes(router, server);
    let router = router.apply_development_router_options();

    info!("HTTP server listening on: {}", args.listen_endpoint);

    axum::Server::bind(&args.listen_endpoint)
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(async move { lgn_cli_utils::wait_for_termination().await.unwrap() })
        .await?;

    Ok(())
}
