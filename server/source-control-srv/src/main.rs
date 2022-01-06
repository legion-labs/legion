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

use anyhow::{Context, Result};
use http::response::Response;
use hyper::body::Body;
use lgn_source_control::{
    commands, BlobStorageUrl, ClearLockRequest, ClountLocksInDomainRequest, CommitExistsRequest,
    CommitToBranchRequest, FindBranchRequest, FindBranchesInLockDomainRequest, FindLockRequest,
    FindLocksInDomainRequest, InsertBranchRequest, InsertCommitRequest, InsertLockRequest,
    InsertWorkspaceRequest, ReadBranchesRequest, ReadCommitRequest, ReadTreeRequest,
    RepositoryQuery, RepositoryUrl, SaveTreeRequest, ServerRequest, UpdateBranchRequest,
};
#[allow(clippy::wildcard_imports)]
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref REPOSITORY_QUERIES: RwLock<HashMap<String, Arc<Box<dyn RepositoryQuery>>>> =
        RwLock::new(HashMap::new());
}

async fn init_remote_repository_req(name: &str) -> Result<String> {
    let repository_url = get_repository_url(name);
    let blob_storage_url = get_blob_storage_url()?;
    let repository_connection =
        commands::create_repository(&repository_url, &Some(blob_storage_url.clone())).await?;

    REPOSITORY_QUERIES.write().unwrap().insert(
        String::from(name),
        Arc::new(repository_connection.repo_query),
    );

    Ok(blob_storage_url.to_string())
}

async fn destroy_repository_req(name: &str) -> Result<String> {
    let repository_url = get_repository_url(name);
    REPOSITORY_QUERIES.write().unwrap().remove(name);
    commands::destroy_repository(&repository_url).await?;

    Ok("".to_string())
}

#[allow(clippy::unnecessary_wraps)]
fn read_blob_storage_spec_req(_name: &str) -> Result<String> {
    let blob_spec: BlobStorageUrl = std::env::var("LEGION_SRC_CTL_BLOB_STORAGE_URI")
        .unwrap()
        .parse()?;

    serde_json::to_string(&blob_spec).context("error serializing blob storage spec")
}

fn get_repository_query(name: &str) -> Result<Arc<Box<dyn RepositoryQuery>>> {
    REPOSITORY_QUERIES
        .read()
        .unwrap()
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("repository not found"))
        .map(Arc::clone)
}

async fn insert_workspace_req(args: &InsertWorkspaceRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    query.insert_workspace(&args.spec).await?;
    Ok(String::from(""))
}

async fn find_branch_req(args: &FindBranchRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    let res = query.find_branch(&args.branch_name).await?;

    serde_json::to_string(&res).context("error formatting find_branch_req result")
}

async fn read_branches_req(args: &ReadBranchesRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    let res = query.read_branches().await?;

    serde_json::to_string(&res).context("error formatting read_branches_req result")
}

async fn find_branches_in_lock_domain(args: &FindBranchesInLockDomainRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    let res = query
        .find_branches_in_lock_domain(&args.lock_domain_id)
        .await?;

    serde_json::to_string(&res).context("error formatting find_branches_in_lock_domain result")
}

async fn read_commit_req(args: &ReadCommitRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    let commit = query.read_commit(&args.commit_id).await?;
    commit.to_json()
}

async fn read_tree_req(args: &ReadTreeRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    let tree = query.read_tree(&args.tree_hash).await?;
    tree.to_json()
}

async fn insert_lock_req(args: &InsertLockRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    query.insert_lock(&args.lock).await?;
    Ok(String::from(""))
}

async fn find_lock_req(args: &FindLockRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    let res = query
        .find_lock(&args.lock_domain_id, &args.canonical_relative_path)
        .await?;

    serde_json::to_string(&res).context("error formatting find_lock result")
}

async fn find_locks_in_domain_req(args: &FindLocksInDomainRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    let res = query.find_locks_in_domain(&args.lock_domain_id).await?;

    serde_json::to_string(&res).context("error formatting find_locks_in_domain result")
}

async fn save_tree_req(args: &SaveTreeRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    query.save_tree(&args.tree, &args.hash).await?;

    Ok(String::from(""))
}

async fn insert_commit_req(args: &InsertCommitRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    query.insert_commit(&args.commit).await?;
    Ok(String::from(""))
}

async fn insert_commit_to_branch_req(args: &CommitToBranchRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    query.commit_to_branch(&args.commit, &args.branch).await?;
    Ok(String::from(""))
}

async fn commit_exists_req(args: &CommitExistsRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    let res = query.commit_exists(&args.commit_id).await?;

    serde_json::to_string(&res).context("error formatting commit_exists result")
}

async fn update_branch_req(args: &UpdateBranchRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    query.update_branch(&args.branch).await?;
    Ok(String::from(""))
}

async fn insert_branch_req(args: &InsertBranchRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    query.insert_branch(&args.branch).await?;
    Ok(String::from(""))
}

async fn clear_lock_req(args: &ClearLockRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    query
        .clear_lock(&args.lock_domain_id, &args.canonical_relative_path)
        .await?;
    Ok(String::from(""))
}

async fn count_locks_in_domain_req(args: &ClountLocksInDomainRequest) -> Result<String> {
    let query = get_repository_query(&args.repo_name)?;
    let res = query.count_locks_in_domain(&args.lock_domain_id).await?;

    serde_json::to_string(&res).context("error formatting count_locks_in_domain_req result")
}

async fn dispatch_request_impl(body: bytes::Bytes) -> Result<String> {
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

#[allow(dead_code)]
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

fn get_repository_url(db_name: &str) -> RepositoryUrl {
    let db_host = std::env::var("LEGION_SRC_CTL_DATABASE_HOST")
        .expect("missing env variable LEGION_SRC_CTL_DATABASE_HOST");
    let db_user = std::env::var("LEGION_SRC_CTL_DATABASE_USERNAME")
        .expect("missing env variable LEGION_SRC_CTL_DATABASE_USERNAME");
    let db_pass = std::env::var("LEGION_SRC_CTL_DATABASE_PASSWORD").unwrap_or_default(); //because it can be empty

    format!("mysql://{}:{}@{}/{}", db_user, db_pass, db_host, db_name)
        .parse()
        .unwrap()
}

fn get_blob_storage_url() -> Result<BlobStorageUrl> {
    std::env::var("LEGION_SRC_CTL_BLOB_STORAGE_URI")
        .unwrap()
        .parse()
}

#[allow(clippy::semicolon_if_nothing_returned)]
#[tokio::main]
async fn main() {
    // TODO: This does not compile anymore but since we will trash it right
    // away once we move to gRPC, let's just comment it.

    //let command_filter = warp::path("lsc")
    //    .and(warp::body::bytes())
    //    .and_then(dispatch_request);

    //let server_addr_str = std::env::var("LEGION_SRC_CTL_SERVER_ADDR")
    //    .expect("missing env variable LEGION_SRC_CTL_SERVER_ADDR");
    //let addr: std::net::SocketAddr = server_addr_str
    //    .parse()
    //    .expect("Error parsing server address");

    //warp::serve(command_filter).run(addr).await;
}
