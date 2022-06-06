#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_variables)]

lgn_online::include_api!();

pub fn register_routes<
    S: permission::Api + space::Api + session::Api + Clone + Send + Sync + 'static,
>(
    router: axum::Router,
    server: S,
) -> axum::Router {
    let router = permission::server::register_routes(router, server.clone());
    let router = space::server::register_routes(router, server.clone());
    session::server::register_routes(router, server)
}
