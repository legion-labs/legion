use http::response::Response;
use hyper::body::Body;
use legion_src_ctl::{sql::*, *};
use warp::Filter;
use Result::Ok;

static mut G_POOL: Option<SqlConnectionPool> = None;

async fn init_remote_repository_req(
    _pool: &SqlConnectionPool,
    name: &str,
) -> Result<String, String> {
    let db_host = std::env::var("LEGION_SRC_CTL_DATABASE_HOST").unwrap();
    let db_user = std::env::var("LEGION_SRC_CTL_DATABASE_USERNAME").unwrap();
    let db_pass = std::env::var("LEGION_SRC_CTL_DATABASE_PASSWORD").unwrap_or_default();
    let s3_uri = std::env::var("LEGION_SRC_CTL_BLOB_STORAGE_URI").unwrap();
    let blob_spec = BlobStorageSpec::S3Uri(s3_uri);
    init_mysql_repo_db(&blob_spec, &db_host, &db_user, &db_pass, name).await?;
    Ok(format!("Created repository {}", name))
}

async fn dispatch_request_impl(
    pool: &SqlConnectionPool,
    body: bytes::Bytes,
) -> Result<String, String> {
    let req = ServerRequest::from_json(std::str::from_utf8(&body).unwrap())?;
    match req {
        ServerRequest::Ping(ping_req) => Ok(format!("Pong from {}", ping_req.specified_uri)),
        ServerRequest::InitRepo(init_req) => init_remote_repository_req(pool, &init_req.name).await,
    }
}

async fn dispatch_request(
    pool: &SqlConnectionPool,
    body: bytes::Bytes,
) -> Result<warp::reply::Response, warp::Rejection> {
    match dispatch_request_impl(pool, body).await {
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

#[tokio::main]
async fn main() {
    let db_host = std::env::var("LEGION_SRC_CTL_DATABASE_HOST")
        .expect("missing env variable LEGION_SRC_CTL_DATABASE_HOST");
    let db_user = std::env::var("LEGION_SRC_CTL_DATABASE_USERNAME")
        .expect("missing env variable LEGION_SRC_CTL_DATABASE_USERNAME");
    let db_pass = std::env::var("LEGION_SRC_CTL_DATABASE_PASSWORD").unwrap_or_default(); //because it can be empty
    let _s3_uri = std::env::var("LEGION_SRC_CTL_BLOB_STORAGE_URI")
        .expect("missing env variable LEGION_SRC_CTL_BLOB_STORAGE_URI");

    let sql_uri = format!("mysql://{}:{}@{}", db_user, db_pass, db_host);

    match make_sql_connection_pool(&sql_uri) {
        Ok(new_pool) => unsafe {
            G_POOL = Some(new_pool);
        },
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    }
    let command_filter =
        warp::path("lsc")
            .and(warp::body::bytes())
            .and_then(|body: bytes::Bytes| async move {
                dispatch_request(unsafe { G_POOL.as_ref().unwrap() }, body).await
            });

    let server_addr_str = std::env::var("LEGION_SRC_CTL_SERVER_ADDR")
        .expect("missing env variable LEGION_SRC_CTL_SERVER_ADDR");
    let addr: std::net::SocketAddr = server_addr_str
        .parse()
        .expect("Error parsing server address");
    warp::serve(command_filter).run(addr).await;
}
