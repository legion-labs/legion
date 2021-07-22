use warp::Filter;

#[tokio::main]
async fn main() {
    let command_filter = warp::path("lsc")
        .and(warp::header("command"))
        .map(|command: String| format!("Pong {}", command));

    warp::serve(command_filter).run(([0, 0, 0, 0], 8080)).await;
}
