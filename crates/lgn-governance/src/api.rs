#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_variables)]

lgn_online::include_api!();

pub fn register_routes<S: space::Api + session::Api + Clone + Send + Sync + 'static>(
    router: axum::Router,
    server: S,
) -> axum::Router {
    let router = session::server::register_routes(router, server.clone());
    space::server::register_routes(router, server)
}
