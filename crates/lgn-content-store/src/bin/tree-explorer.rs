//! A content-store index trees explorer.
//!
use std::{net::SocketAddr, sync::Arc};

use axum::{
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use http::StatusCode;
use lgn_content_store::{
    indexing::{
        tree_visit, IndexKey, IndexKeyDisplayFormat, IndexableResource, JsonVisitor,
        ResourceWriter, StaticIndexer, Tree, TreeIdentifier, TreeLeafNode, TreeWriter,
    },
    Provider, Result,
};
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::{async_span_scope, error, info, LevelFilter};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'd', long = "debug")]
    debug: bool,

    #[clap(
        short = 'l',
        long = "listen-endpoint",
        default_value = "127.0.0.1:3000"
    )]
    listen_endpoint: SocketAddr,

    #[clap(short = 'f', long = "display-format", default_value = "hex")]
    display_format: IndexKeyDisplayFormat,
}

struct State {
    provider: Provider,
    indexer: StaticIndexer,
    tree_id: Mutex<TreeIdentifier>,
    display_format: IndexKeyDisplayFormat,
}

impl State {
    async fn new(display_format: IndexKeyDisplayFormat) -> Result<Self> {
        let provider = Provider::new_in_memory();
        let mut indexer = StaticIndexer::new(4);
        indexer.set_layer_constraints(2, 4);
        let tree_id = Mutex::new(provider.write_tree(&Tree::default()).await?);

        Ok(Self {
            provider,
            indexer,
            tree_id,
            display_format,
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    let _telemetry_guard = TelemetryGuardBuilder::default()
        .with_local_sink_enabled(args.debug)
        .with_local_sink_max_level(LevelFilter::Debug)
        .build();

    async_span_scope!("lgn-content-store-srv::main");

    //let provider = Config::load_and_instantiate_persistent_provider()
    //    .await
    //    .map_err(|err| anyhow::anyhow!("failed to create content provider: {}", err))?;
    info!(
        "Using tree with index key display format: {}",
        args.display_format
    );

    let state = Arc::new(State::new(args.display_format).await?);

    // build our application with a single route
    let app = Router::new()
        .route(
            "/nodes",
            post({
                let state = Arc::clone(&state);
                move |body| add_node(body, state)
            }),
        )
        .route(
            "/nodes",
            get({
                let state = Arc::clone(&state);
                move || graph(state)
            }),
        )
        .route(
            "/style.css",
            get(|| async { axum::response::Html(include_str!("tree-explorer/html/style.css")) }),
        )
        .route(
            "/",
            get(|| async { axum::response::Html(include_str!("tree-explorer/html/index.html")) }),
        );

    info!("Listening on {}", args.listen_endpoint);

    axum::Server::bind(&args.listen_endpoint)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddNode {
    index_key: String,
    data: Data,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Data(serde_json::Value);

impl IndexableResource for Data {}

async fn add_node(
    Json(body): Json<AddNode>,
    state: Arc<State>,
) -> Result<Json<impl Serialize>, StatusCode> {
    let mut tree_id = state.tree_id.lock().await;

    let resource_id = state
        .provider
        .write_resource(&body.data)
        .await
        .map_err(|err| {
            error!("failed to write resource: {}", err);

            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let leaf_node = TreeLeafNode::Resource(resource_id);
    let index_key = IndexKey::parse(state.display_format, &body.index_key).map_err(|err| {
        error!("failed to parse index key: {}", err);

        StatusCode::BAD_REQUEST
    })?;

    *tree_id = state
        .indexer
        .add_leaf(&state.provider, &*tree_id, &index_key, leaf_node)
        .await
        .map_err(|err| match err {
            lgn_content_store::indexing::Error::IndexTreeLeafNodeAlreadyExists(..) => {
                StatusCode::CONFLICT
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(Json(
        tree_visit(
            &state.provider,
            &tree_id,
            JsonVisitor::new(state.display_format),
        )
        .await
        .map_err(|err| {
            error!("failed to visit tree: {}", err);

            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_result(),
    ))
}

async fn graph(state: Arc<State>) -> Result<Json<impl Serialize>, StatusCode> {
    let tree_id = state.tree_id.lock().await.clone();

    Ok(Json(
        tree_visit(
            &state.provider,
            &tree_id,
            JsonVisitor::new(state.display_format),
        )
        .await
        .map_err(|err| {
            error!("failed to visit tree: {}", err);

            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_result(),
    ))
}
