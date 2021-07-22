use warp::Filter;

#[tokio::main]
async fn main() {
    let command_filter = warp::path("lsc")
        .and(warp::body::bytes())
        .map(|body: bytes::Bytes| format!("Pong {}", std::str::from_utf8(&body).unwrap()));

    warp::serve(command_filter).run(([0, 0, 0, 0], 8080)).await;
}
