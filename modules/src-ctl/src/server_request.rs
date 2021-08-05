use crate::{Lock, Workspace};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PingRequest {
    pub specified_uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitRepositoryRequest {
    pub repo_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DestroyRepositoryRequest {
    pub repo_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadBlobStorageSpecRequest {
    pub repo_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InsertWorkspaceRequest {
    pub repo_name: String,
    pub spec: Workspace,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadBranchRequest {
    pub repo_name: String,
    pub branch_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadCommitRequest {
    pub repo_name: String,
    pub commit_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadTreeRequest {
    pub repo_name: String,
    pub tree_hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InsertLockRequest {
    pub repo_name: String,
    pub lock: Lock,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FindLockRequest {
    pub repo_name: String,
    pub lock_domain_id: String,
    pub canonical_relative_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerRequest {
    InitRepo(InitRepositoryRequest),
    DestroyRepo(DestroyRepositoryRequest),
    InsertWorkspace(InsertWorkspaceRequest),
    Ping(PingRequest),
    ReadBlobStorageSpec(ReadBlobStorageSpecRequest),
    ReadBranch(ReadBranchRequest),
    ReadCommit(ReadCommitRequest),
    ReadTree(ReadTreeRequest),
    InsertLock(InsertLockRequest),
    FindLock(FindLockRequest),
}

impl ServerRequest {
    pub fn to_json(&self) -> Result<String, String> {
        match serde_json::to_string(&self) {
            Ok(json) => Ok(json),
            Err(e) => Err(format!("Error formatting server request: {}", e)),
        }
    }

    pub fn from_json(contents: &str) -> Result<Self, String> {
        let parsed: serde_json::Result<Self> = serde_json::from_str(contents);
        match parsed {
            Ok(req) => Ok(req),
            Err(e) => Err(format!("Error parsing server request: {}", e)),
        }
    }
}

pub async fn execute_request(http_url: &str, request: &ServerRequest) -> Result<String, String> {
    let client = reqwest::Client::new();
    match client.get(http_url).body(request.to_json()?).send().await {
        Ok(resp) => {
            let status = resp.status();
            if !status.is_success() {
                return Err(format!(
                    "Request {} failed with status {}\n{}",
                    http_url,
                    status,
                    resp.text().await.unwrap_or_default()
                ));
            }
            match resp.text().await {
                Ok(body) => Ok(body),
                Err(e) => Err(format!("Error reading response body: {}", e)),
            }
        }
        Err(e) => Err(format!("Error contacting server: {}", e)),
    }
}
