use http::response::Response;
use hyper::body::Body;
use legion_src_ctl::*;
use warp::Filter;

static mut G_POOL: Option<SqlConnectionPool> = None;

fn dispatch_request_impl(_pool: &SqlConnectionPool, body: bytes::Bytes) -> Result<String, String> {
    let req = ServerRequest::from_json(std::str::from_utf8(&body).unwrap())?;
    Ok(format!("Pong {:?}", req))
}

fn dispatch_request(pool: &SqlConnectionPool, body: bytes::Bytes) -> warp::reply::Response {
    match dispatch_request_impl(pool, body) {
        Ok(body) => Response::builder().body(Body::from(body)).unwrap(),
        Err(e) => {
            let message = format!("Error processing request: {}", e);
            Response::builder()
                .status(500)
                .body(Body::from(message))
                .unwrap()
        }
    }
}

#[tokio::main]
async fn main() {
    let sql_uri = std::env::var("LEGION_SRC_CTL_DATABASE_SERVER_URI")
        .expect("missing env variable LEGION_SRC_CTL_DATABASE_SERVER_URI");

    match make_sql_connection_pool(&sql_uri) {
        Ok(new_pool) => unsafe {
            G_POOL = Some(new_pool);
        },
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    }
    let command_filter = warp::path("lsc")
        .and(warp::body::bytes())
        .map(|body: bytes::Bytes| dispatch_request(unsafe { G_POOL.as_ref().unwrap() }, body));

    let server_addr_str = std::env::var("LEGION_SRC_CTL_SERVER_ADDR")
        .expect("missing env variable LEGION_SRC_CTL_SERVER_ADDR");
    let addr: std::net::SocketAddr = server_addr_str
        .parse()
        .expect("Error parsing server address");
    warp::serve(command_filter).run(addr).await;
}
