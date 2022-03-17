//! A content-store implementation that stores immutable assets in a efficient
//! and cachable manner.

use bytesize::ByteSize;
use clap::Parser;
use http::{header, Method};
use lgn_cli_utils::termination_handler::AsyncTerminationHandler;
use lgn_content_store2::{AwsDynamoDbProvider, AwsS3Provider, GrpcService};
use lgn_content_store_proto::content_store_server::ContentStoreServer;
use lgn_online::authentication::{jwt::RequestAuthorizer, UserInfo};
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::prelude::*;
use std::{net::SocketAddr, time::Duration};
use tonic::transport::Server;
use tower_http::{
    auth::RequireAuthorizationLayer,
    cors::{CorsLayer, Origin},
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

    #[clap(long, default_value = "128KiB")]
    size_treshold: ByteSize,

    #[clap(long, default_value = "s3://legionlabs-content-store/")]
    s3_bucket: String,

    #[clap(long, default_value = "legionlabs-content-store")]
    dynamodb_table_name: String,
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

    span_scope!("lgn-content-store-srv::main");

    let cors = CorsLayer::new()
        .allow_origin(Origin::list(args.origins))
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

    let signature_validation_config = lgn_online::authentication::SignatureValidationConfig::new()
        .map_err(|err| anyhow::anyhow!("failed to load signature validation config: {}", err))?;

    let validation = signature_validation_config.validation().await?;

    let auth_layer =
        RequireAuthorizationLayer::custom(RequestAuthorizer::<UserInfo, _, _>::new(validation));

    let layer = tower::ServiceBuilder::new() //todo: compose with cors layer
        .layer(auth_layer)
        .layer(cors)
        .into_inner();

    let mut server = Server::builder().accept_http1(true).layer(layer);

    // Hardcode AWS providers for now.
    info!("Using AWS S3 bucket: {}", args.s3_bucket);
    info!("Using AWS DynamoDB table: {}", args.dynamodb_table_name);

    let aws_s3_url = args.s3_bucket.parse().unwrap();
    let aws_s3_provider = AwsS3Provider::new(aws_s3_url).await;
    let aws_dynamo_db_provider = AwsDynamoDbProvider::new(args.dynamodb_table_name).await;
    let grpc_service = GrpcService::new(
        aws_dynamo_db_provider,
        aws_s3_provider,
        args.size_treshold
            .as_u64()
            .try_into()
            .expect("size_treshold is too big"),
    );

    let service = ContentStoreServer::new(grpc_service);
    let server = server.add_service(tonic_web::enable(service));

    let handler = AsyncTerminationHandler::new()?;

    info!("Listening on {}", args.listen_endpoint);

    tokio::select! {
        _ = handler.wait() => Ok(()),
        res = server.serve(args.listen_endpoint) => res.map_err(Into::into),
    }
}
