use http::response::Response;
use hyper::body::Body;
use legion_src_ctl::sql_repository_query::SqlRepositoryQuery;
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
    let db_uri = format!("{}/{}", db_server_uri, name);
    let pool = init_mysql_repo_db(&blob_spec, &db_uri).await?;
    POOLS.write().unwrap().insert(String::from(name), pool);
    Ok(format!("Created repository {}", name))
}

async fn destroy_repository_req(name: &str) -> Result<String,String>{
    let db_server_uri = get_sql_uri();
    let db_uri = format!("{}/{}", db_server_uri, name);
    POOLS.write().unwrap().remove(name);
    sql::drop_database(&db_uri)?;
    Ok(format!("Dropped repository {}", name))
}

fn read_blob_storage_spec_req(_name: &str) -> Result<String, String> {
    let s3_uri = std::env::var("LEGION_SRC_CTL_BLOB_STORAGE_URI").unwrap();
    let blob_spec = BlobStorageSpec::S3Uri(s3_uri);
    Ok(blob_spec.to_json())
}

async fn get_connection_pool(repo_name: &str) -> Result<Arc<SqlConnectionPool>, String> {
    {
        let pool_read = POOLS.read().unwrap();
        if let Some(p) = pool_read.get(repo_name) {
            return Ok(p.clone());
        }
    }

    let db_server_uri = get_sql_uri();
    let repo_uri = format!("{}/{}", db_server_uri, repo_name);
    let p = Arc::new(SqlConnectionPool::new(&repo_uri).await?);
    POOLS
        .write()
        .unwrap()
        .insert(String::from(repo_name), p.clone());
    Ok(p)
}

async fn insert_workspace_req(repo_name: &str, spec: &Workspace) -> Result<String, String> {
    let query = SqlRepositoryQuery::new(get_connection_pool(repo_name).await?);
    query.insert_workspace(spec).await?;
    Ok(String::from(""))
}

async fn read_branch_req(repo_name: &str, branch_name: &str) -> Result<String, String> {
    let query = SqlRepositoryQuery::new(get_connection_pool(repo_name).await?);
    let branch = query.read_branch(branch_name).await?;
    branch.to_json()
}

async fn read_commit_req(repo_name: &str, commit_id: &str) -> Result<String, String> {
    let query = SqlRepositoryQuery::new(get_connection_pool(repo_name).await?);
    let commit = query.read_commit(commit_id).await?;
    commit.to_json()
}

async fn read_tree_req(repo_name: &str, tree_hash: &str) -> Result<String, String> {
    let query = SqlRepositoryQuery::new(get_connection_pool(repo_name).await?);
    let tree = query.read_tree(tree_hash).await?;
    tree.to_json()
}

async fn dispatch_request_impl(body: bytes::Bytes) -> Result<String, String> {
    let req = ServerRequest::from_json(std::str::from_utf8(&body).unwrap())?;
    println!("{:?}", req);
    match req {
        ServerRequest::Ping(req) => Ok(format!("Pong from {}", req.specified_uri)),
        ServerRequest::InitRepo(req) => init_remote_repository_req(&req.repo_name).await,
        ServerRequest::DestroyRepo(req) => destroy_repository_req(&req.repo_name).await,
        ServerRequest::ReadBlobStorageSpec(req) => read_blob_storage_spec_req(&req.repo_name),
        ServerRequest::InsertWorkspace(req) => {
            insert_workspace_req(&req.repo_name, &req.spec).await
        }
        ServerRequest::ReadBranch(req) => read_branch_req(&req.repo_name, &req.branch_name).await,
        ServerRequest::ReadCommit(req) => read_commit_req(&req.repo_name, &req.commit_id).await,
        ServerRequest::ReadTree(req) => read_tree_req(&req.repo_name, &req.tree_hash).await,
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
