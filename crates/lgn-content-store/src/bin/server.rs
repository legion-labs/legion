//! A content-store implementation that stores immutable assets in a efficient
//! and cachable manner.

use axum::Router;
use bytesize::ByteSize;
use clap::Parser;
use lgn_content_store::{
    content_store::server, AddressProviderConfig, AliasProviderConfig, ApiProviderSet,
    ContentProviderConfig, DataSpace, Result, Server,
};
use lgn_online::server::RouterExt;
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::prelude::*;
use serde::Deserialize;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    debug: bool,

    #[clap(short, long, default_value = "0.0.0.0:5000")]
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
    providers: HashMap<DataSpace, ProviderSetConfig>,
}

impl Config {
    async fn instantiate_providers(&self) -> Result<HashMap<DataSpace, ApiProviderSet>> {
        let mut result = HashMap::new();

        for (data_space, provider_set_config) in &self.providers {
            let provider_set = provider_set_config.instantiate().await?;
            result.insert(data_space.clone(), provider_set);
        }

        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ProviderSetConfig {
    content_provider: ContentProviderConfig,
    alias_provider: AliasProviderConfig,
    address_provider: AddressProviderConfig,

    #[serde(default = "ProviderSetConfig::default_size_threshold")]
    size_treshold: ByteSize,
}

impl ProviderSetConfig {
    fn default_size_threshold() -> ByteSize {
        "128KiB".parse().unwrap()
    }

    async fn instantiate(&self) -> Result<ApiProviderSet> {
        Ok(ApiProviderSet {
            content_provider: self.content_provider.instantiate().await?,
            alias_provider: self.alias_provider.instantiate().await?,
            content_address_provider: self.address_provider.instantiate().await?,
            size_threshold: self
                .size_treshold
                .as_u64()
                .try_into()
                .expect("size_threshold is too large"),
        })
    }
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

    async_span_scope!("lgn-content-store-srv::main");

    let config: Config = lgn_config::get("content_store.server")?
        .ok_or_else(|| anyhow::anyhow!("no configuration was found for `content_store.server`"))?;

    let providers = config.instantiate_providers().await?;

    if providers.is_empty() {
        return Err(anyhow::anyhow!("no providers were configured"));
    }

    info!("Now listing configured {} provider(s)...", providers.len());

    for (i, (data_space, provider_set)) in providers.iter().enumerate() {
        info!(
            "{}: {} - provider: {} - address provider: {} - size threshold: {} - alias provider: {}",
            i,
            data_space,
            provider_set.content_provider,
            provider_set.content_address_provider,
            provider_set.size_threshold,
            provider_set.alias_provider,
        );
    }

    let server = Arc::new(Server::new(providers));
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
