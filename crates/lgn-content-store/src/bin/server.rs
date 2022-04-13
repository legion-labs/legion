//! A content-store implementation that stores immutable assets in a efficient
//! and cachable manner.

use bytesize::ByteSize;
use clap::Parser;
use http::{header, Method};
use lgn_cli_utils::termination_handler::AsyncTerminationHandler;
use lgn_content_store::{
    AddressProviderConfig, DataSpace, GrpcProviderSet, GrpcService, ProviderConfig, Result,
};
use lgn_content_store_proto::content_store_server::ContentStoreServer;
use lgn_online::authentication::{jwt::RequestAuthorizer, UserInfo};
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::prelude::*;
use serde::Deserialize;
use std::{collections::HashMap, net::SocketAddr, time::Duration};
use tonic::transport::Server;
use tower_http::{
    auth::RequireAuthorizationLayer,
    cors::{AllowOrigin, CorsLayer},
};

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
    async fn instantiate_providers(&self) -> Result<HashMap<DataSpace, GrpcProviderSet>> {
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
    provider: ProviderConfig,
    address_provider: AddressProviderConfig,

    #[serde(default = "ProviderSetConfig::default_size_threshold")]
    size_treshold: ByteSize,
}

impl ProviderSetConfig {
    fn default_size_threshold() -> ByteSize {
        "128KiB".parse().unwrap()
    }

    async fn instantiate(&self) -> Result<GrpcProviderSet> {
        Ok(GrpcProviderSet {
            provider: self.provider.instantiate().await?,
            address_provider: self.address_provider.instantiate().await?,
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

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(args.origins))
        .allow_credentials(true)
        .max_age(Duration::from_secs(60 * 60))
        .allow_headers(vec![
            header::ACCEPT,
            header::ACCEPT_LANGUAGE,
            header::AUTHORIZATION,
            header::CONTENT_LANGUAGE,
            header::CONTENT_TYPE,
            header::HeaderName::from_static("x-grpc-web"),
        ])
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::HEAD,
            Method::OPTIONS,
            Method::CONNECT,
        ])
        .expose_headers(tower_http::cors::Any {});

    let validation = lgn_online::Config::load()?
        .signature_validation
        .instantiate_validation()
        .await?;

    let auth_layer =
        RequireAuthorizationLayer::custom(RequestAuthorizer::<UserInfo, _, _>::new(validation));

    let layer = tower::ServiceBuilder::new() //todo: compose with cors layer
        .layer(auth_layer)
        .layer(cors)
        .into_inner();

    let mut server = Server::builder().accept_http1(true).layer(layer);

    let providers = config.instantiate_providers().await?;

    if providers.is_empty() {
        return Err(anyhow::anyhow!("no providers were configured"));
    }

    info!("Now listing configured {} provider(s)...", providers.len());

    for (i, (data_space, provider_set)) in providers.iter().enumerate() {
        info!(
            "{}: {} - provider: {} - address provider: {} - size threshold: {}",
            i,
            data_space,
            provider_set.provider,
            provider_set.address_provider,
            provider_set.size_threshold
        );
    }

    let grpc_service = GrpcService::new(providers);

    let service = ContentStoreServer::new(grpc_service);
    let server = server.add_service(tonic_web::enable(service));

    let handler = AsyncTerminationHandler::new()?;

    info!("Listening on {}", args.listen_endpoint);

    tokio::select! {
        _ = handler.wait() => Ok(()),
        res = server.serve(args.listen_endpoint) => res.map_err(Into::into),
    }
}
