//! The Governance server executable.

use std::{net::SocketAddr, sync::Arc};

use axum::Router;
use clap::Parser;
use lgn_governance::Server;
use lgn_online::server::RouterExt;
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::{async_span_scope, debug, info, LevelFilter};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    debug: bool,

    #[clap(short, long, default_value = "0.0.0.0:5000")]
    listen_endpoint: SocketAddr,

    #[clap(
        long,
        env,
        default_value = "mysql://root@localhost:3306/lgn_governance"
    )]
    database_url: String,

    #[clap(
        long,
        default_value = "",
        help = "The list of origins that are allowed to make requests, for CORS"
    )]
    origins: Vec<http::HeaderValue>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    let _telemetry_guard = TelemetryGuardBuilder::default()
        .with_local_sink_max_level(if args.debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .build();

    async_span_scope!("lgn-governance-srv::main");

    if args.debug {
        debug!("Starting in debug mode");
    } else {
        info!("Starting in production mode");
    }

    // Only display this sensitive information in debug mode.
    if args.debug {
        info!("Connecting to MySQL: {}", args.database_url);
    } else {
        info!("Connecting to MySQL");
    }

    let server = Arc::new(Server::new(&args.database_url).await?);
    let router = lgn_governance::register_routes(Router::new(), server);
    let router = router.apply_development_router_options();

    info!("HTTP server listening on: {}", args.listen_endpoint);

    axum::Server::bind(&args.listen_endpoint)
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(async move { lgn_cli_utils::wait_for_termination().await.unwrap() })
        .await?;

    Ok(())
}
