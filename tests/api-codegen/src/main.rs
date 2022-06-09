//! Testing server for the api codegen.
//!
// crate-specific lint exceptions:
// #![allow()]

use std::net::SocketAddr;

use api_codegen::{api::cars::server, api_impl::ApiImpl};
use axum::Router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api = ApiImpl::default();
    let router = server::register_routes(Router::new(), api);

    let addr = "127.0.0.1:3000".parse().unwrap();
    println!("Server listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}
