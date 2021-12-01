use std::net::SocketAddr;

use axum::{
    body::{boxed, Body, BoxBody},
    http::{Request, Response, StatusCode, Uri},
    routing::get,
    Router, Server,
};
use tower::ServiceExt;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let app = Router::new().nest("/", get(handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("Listening on http://localhost:3000");

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let res = get_static_file(uri.clone()).await?;

    if res.status() == StatusCode::NOT_FOUND {
        // When a file is not found we just default to / (i.e. index.html)
        get_static_file("/".parse().unwrap()).await
    } else {
        Ok(res)
    }
}

async fn get_static_file(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

    match ServeDir::new("./frontend/dist").oneshot(req).await {
        Ok(res) => Ok(res.map(boxed)),
        Err(error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("An error occured: {}", error),
        )),
    }
}
