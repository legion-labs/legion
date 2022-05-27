//! Testing server for the api codegen.
//!
// crate-specific lint exceptions:
// #![allow()]

use api_codegen::{api_impl::ApiImpl, cars::server};
use axum::Router;
use lgn_online::server::RouterExt;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let api = ApiImpl::default();

    let router = Router::new().apply_development_router_options();
    let router = server::register_routes(router, api);

    let addr = "127.0.0.1:3000".parse().unwrap();
    println!("Server listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
