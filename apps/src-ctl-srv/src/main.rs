use http::response::Response;
use hyper::body::Body;
use legion_src_ctl::{sql::*, *};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use warp::Filter;
use Result::Ok;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref POOLS: RwLock<HashMap<String, Arc<SqlConnectionPool>>> = RwLock::new(HashMap::new());
}

async fn init_remote_repository_req(name: &str) -> Result<String, String> {
    let s3_uri = std::env::var("LEGION_SRC_CTL_BLOB_STORAGE_URI").unwrap();
    let blob_spec = BlobStorageSpec::S3Uri(s3_uri);
    let db_server_uri = get_sql_uri();
    let pool = init_mysql_repo_db(&blob_spec, &db_server_uri, name).await?;
    POOLS.write().unwrap().insert(String::from(name), pool);
    Ok(format!("Created repository {}", name))
}

async fn dispatch_request_impl(body: bytes::Bytes) -> Result<String, String> {
    let req = ServerRequest::from_json(std::str::from_utf8(&body).unwrap())?;
    match req {
        ServerRequest::Ping(ping_req) => Ok(format!("Pong from {}", ping_req.specified_uri)),
        ServerRequest::InitRepo(init_req) => init_remote_repository_req(&init_req.name).await,
    }
}

async fn dispatch_request(body: bytes::Bytes) -> Result<warp::reply::Response, warp::Rejection> {
    match dispatch_request_impl(body).await {
        Ok(body) => Ok(Response::builder().body(Body::from(body)).unwrap()),
        Err(e) => {
            let message = format!("Error processing request: {}", e);
            Ok(Response::builder()
                .status(500)
                .body(Body::from(message))
                .unwrap())
        }
    }
}

fn get_sql_uri() -> String {
    let db_host = std::env::var("LEGION_SRC_CTL_DATABASE_HOST")
        .expect("missing env variable LEGION_SRC_CTL_DATABASE_HOST");
    let db_user = std::env::var("LEGION_SRC_CTL_DATABASE_USERNAME")
        .expect("missing env variable LEGION_SRC_CTL_DATABASE_USERNAME");
    let db_pass = std::env::var("LEGION_SRC_CTL_DATABASE_PASSWORD").unwrap_or_default(); //because it can be empty
    format!("mysql://{}:{}@{}", db_user, db_pass, db_host)
}

#[tokio::main]
async fn main() {
    let _s3_uri = std::env::var("LEGION_SRC_CTL_BLOB_STORAGE_URI")
        .expect("missing env variable LEGION_SRC_CTL_BLOB_STORAGE_URI");
    let _db_server_uri = get_sql_uri();

    let command_filter = warp::path("lsc")
        .and(warp::body::bytes())
        .and_then(dispatch_request);

    let server_addr_str = std::env::var("LEGION_SRC_CTL_SERVER_ADDR")
        .expect("missing env variable LEGION_SRC_CTL_SERVER_ADDR");
    let addr: std::net::SocketAddr = server_addr_str
        .parse()
        .expect("Error parsing server address");
    warp::serve(command_filter).run(addr).await;
}
