//! Legion Source Control Server
//!
//! TODO: write documentation.

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow()]

use http::response::Response;
use hyper::body::Body;
use legion_source_control::sql_repository_query::{Databases, SqlRepositoryQuery};
#[allow(clippy::wildcard_imports)]
use legion_source_control::{sql::SqlConnectionPool, *};
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

async fn destroy_repository_req(name: &str) -> Result<String, String> {
    let db_server_uri = get_sql_uri();
    let db_uri = format!("{}/{}", db_server_uri, name);
    POOLS.write().unwrap().remove(name);
    sql::drop_database(&db_uri).await?;
    Ok(format!("Dropped repository {}", name))
}

#[allow(clippy::unnecessary_wraps)]
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

async fn get_sql_query_interface(repo_name: &str) -> Result<SqlRepositoryQuery, String> {
    Ok(SqlRepositoryQuery::new(
        get_connection_pool(repo_name).await?,
        Databases::Mysql,
    ))
}

async fn insert_workspace_req(args: &InsertWorkspaceRequest) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    query.insert_workspace(&args.spec).await?;
    Ok(String::from(""))
}

async fn find_branch_req(args: &FindBranchRequest) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    let res = query.find_branch(&args.branch_name).await?;
    match serde_json::to_string(&res) {
        Ok(json) => Ok(json),
        Err(e) => Err(format!("Error formatting find_branch_req result: {}", e)),
    }
}

async fn read_branches_req(args: &ReadBranchesRequest) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    let res = query.read_branches().await?;
    match serde_json::to_string(&res) {
        Ok(json) => Ok(json),
        Err(e) => Err(format!("Error formatting read_branches_req result: {}", e)),
    }
}

async fn find_branches_in_lock_domain(
    args: &FindBranchesInLockDomainRequest,
) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    let res = query
        .find_branches_in_lock_domain(&args.lock_domain_id)
        .await?;
    match serde_json::to_string(&res) {
        Ok(json) => Ok(json),
        Err(e) => Err(format!(
            "Error formatting find_branches_in_lock_domain result: {}",
            e
        )),
    }
}

async fn read_commit_req(args: &ReadCommitRequest) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    let commit = query.read_commit(&args.commit_id).await?;
    commit.to_json()
}

async fn read_tree_req(args: &ReadTreeRequest) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    let tree = query.read_tree(&args.tree_hash).await?;
    tree.to_json()
}

async fn insert_lock_req(args: &InsertLockRequest) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    query.insert_lock(&args.lock).await?;
    Ok(String::from(""))
}

async fn find_lock_req(args: &FindLockRequest) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    let res = query
        .find_lock(&args.lock_domain_id, &args.canonical_relative_path)
        .await?;
    match serde_json::to_string(&res) {
        Ok(json) => Ok(json),
        Err(e) => Err(format!("Error formatting find_lock result: {}", e)),
    }
}

async fn find_locks_in_domain_req(args: &FindLocksInDomainRequest) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    let res = query.find_locks_in_domain(&args.lock_domain_id).await?;
    match serde_json::to_string(&res) {
        Ok(json) => Ok(json),
        Err(e) => Err(format!(
            "Error formatting find_locks_in_domain result: {}",
            e
        )),
    }
}

async fn save_tree_req(args: &SaveTreeRequest) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    query.save_tree(&args.tree, &args.hash).await?;
    Ok(String::from(""))
}

async fn insert_commit_req(args: &InsertCommitRequest) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    query.insert_commit(&args.commit).await?;
    Ok(String::from(""))
}

async fn insert_commit_to_branch_req(args: &CommitToBranchRequest) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    query.commit_to_branch(&args.commit, &args.branch).await?;
    Ok(String::from(""))
}

async fn commit_exists_req(args: &CommitExistsRequest) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    let res = query.commit_exists(&args.commit_id).await?;
    match serde_json::to_string(&res) {
        Ok(json) => Ok(json),
        Err(e) => Err(format!("Error formatting commit_exists_req result: {}", e)),
    }
}

async fn update_branch_req(args: &UpdateBranchRequest) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    query.update_branch(&args.branch).await?;
    Ok(String::from(""))
}

async fn insert_branch_req(args: &InsertBranchRequest) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    query.insert_branch(&args.branch).await?;
    Ok(String::from(""))
}

async fn clear_lock_req(args: &ClearLockRequest) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    query
        .clear_lock(&args.lock_domain_id, &args.canonical_relative_path)
        .await?;
    Ok(String::from(""))
}

async fn count_locks_in_domain_req(args: &ClountLocksInDomainRequest) -> Result<String, String> {
    let query = get_sql_query_interface(&args.repo_name).await?;
    let res = query.count_locks_in_domain(&args.lock_domain_id).await?;
    match serde_json::to_string(&res) {
        Ok(json) => Ok(json),
        Err(e) => Err(format!(
            "Error formatting count_locks_in_domain_req result: {}",
            e
        )),
    }
}

async fn dispatch_request_impl(body: bytes::Bytes) -> Result<String, String> {
    let req = ServerRequest::from_json(std::str::from_utf8(&body).unwrap())?;
    println!("{:?}", req);
    match req {
        ServerRequest::Ping(req) => Ok(format!("Pong from {}", req.specified_uri)),
        ServerRequest::InitRepo(req) => init_remote_repository_req(&req.repo_name).await,
        ServerRequest::DestroyRepo(req) => destroy_repository_req(&req.repo_name).await,
        ServerRequest::ReadBlobStorageSpec(req) => read_blob_storage_spec_req(&req.repo_name),
        ServerRequest::InsertWorkspace(req) => insert_workspace_req(&req).await,
        ServerRequest::FindBranch(req) => find_branch_req(&req).await,
        ServerRequest::ReadBranches(req) => read_branches_req(&req).await,
        ServerRequest::FindBranchesInLockDomain(req) => find_branches_in_lock_domain(&req).await,
        ServerRequest::ReadCommit(req) => read_commit_req(&req).await,
        ServerRequest::ReadTree(req) => read_tree_req(&req).await,
        ServerRequest::InsertLock(req) => insert_lock_req(&req).await,
        ServerRequest::FindLock(req) => find_lock_req(&req).await,
        ServerRequest::FindLocksInDomain(req) => find_locks_in_domain_req(&req).await,
        ServerRequest::SaveTree(req) => save_tree_req(&req).await,
        ServerRequest::InsertCommit(req) => insert_commit_req(&req).await,
        ServerRequest::CommitToBranch(req) => insert_commit_to_branch_req(&req).await,
        ServerRequest::CommitExists(req) => commit_exists_req(&req).await,
        ServerRequest::UpdateBranch(req) => update_branch_req(&req).await,
        ServerRequest::InsertBranch(req) => insert_branch_req(&req).await,
        ServerRequest::ClearLock(req) => clear_lock_req(&req).await,
        ServerRequest::ClountLocksInDomain(req) => count_locks_in_domain_req(&req).await,
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

#[allow(clippy::semicolon_if_nothing_returned)]
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
