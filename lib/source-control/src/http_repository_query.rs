use async_trait::async_trait;

use crate::{
    execute_request, BlobStorageSpec, Branch, ClearLockRequest, ClountLocksInDomainRequest, Commit,
    CommitExistsRequest, CommitToBranchRequest, FindBranchRequest, FindBranchesInLockDomainRequest,
    FindLockRequest, FindLocksInDomainRequest, InsertBranchRequest, InsertCommitRequest,
    InsertLockRequest, InsertWorkspaceRequest, Lock, ReadBlobStorageSpecRequest,
    ReadBranchesRequest, ReadCommitRequest, ReadTreeRequest, RepositoryQuery, SaveTreeRequest,
    ServerRequest, Tree, UpdateBranchRequest, Workspace,
};

// access to repository metadata through a web server
pub struct HTTPRepositoryQuery {
    url: String,
    repo_name: String,
    client: reqwest::Client,
}

impl HTTPRepositoryQuery {
    pub fn new(url: String, repo_name: String) -> Result<Self, String> {
        Ok(Self {
            url,
            repo_name,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl RepositoryQuery for HTTPRepositoryQuery {
    async fn insert_workspace(&self, spec: &Workspace) -> Result<(), String> {
        let request = ServerRequest::InsertWorkspace(InsertWorkspaceRequest {
            repo_name: self.repo_name.clone(),
            spec: spec.clone(),
        });
        let _resp = execute_request(&self.client, &self.url, &request).await?;
        Ok(())
    }

    async fn read_branch(&self, name: &str) -> Result<Branch, String> {
        match self.find_branch(name).await {
            Ok(Some(obj)) => Ok(obj),
            Ok(None) => Err(format!("branch {} not found", name)),
            Err(e) => Err(format!("Error in read_branch: {}", e)),
        }
    }

    async fn find_branch(&self, name: &str) -> Result<Option<Branch>, String> {
        let request = ServerRequest::FindBranch(FindBranchRequest {
            repo_name: self.repo_name.clone(),
            branch_name: String::from(name),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;
        let parsed: serde_json::Result<Option<Branch>> = serde_json::from_str(&resp);
        match parsed {
            Ok(obj) => Ok(obj),
            Err(e) => Err(format!("Error parsing response: {}", e)),
        }
    }

    async fn read_branches(&self) -> Result<Vec<Branch>, String> {
        let request = ServerRequest::ReadBranches(ReadBranchesRequest {
            repo_name: self.repo_name.clone(),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;
        let parsed: serde_json::Result<Vec<Branch>> = serde_json::from_str(&resp);
        match parsed {
            Ok(obj) => Ok(obj),
            Err(e) => Err(format!("Error parsing response: {}", e)),
        }
    }

    async fn insert_branch(&self, branch: &Branch) -> Result<(), String> {
        let request = ServerRequest::InsertBranch(InsertBranchRequest {
            repo_name: self.repo_name.clone(),
            branch: branch.clone(),
        });
        let _resp = execute_request(&self.client, &self.url, &request).await?;
        Ok(())
    }

    async fn update_branch(&self, branch: &Branch) -> Result<(), String> {
        let request = ServerRequest::UpdateBranch(UpdateBranchRequest {
            repo_name: self.repo_name.clone(),
            branch: branch.clone(),
        });
        let _resp = execute_request(&self.client, &self.url, &request).await?;
        Ok(())
    }

    async fn find_branches_in_lock_domain(
        &self,
        lock_domain_id: &str,
    ) -> Result<Vec<Branch>, String> {
        let request = ServerRequest::FindBranchesInLockDomain(FindBranchesInLockDomainRequest {
            repo_name: self.repo_name.clone(),
            lock_domain_id: String::from(lock_domain_id),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;
        let parsed: serde_json::Result<Vec<Branch>> = serde_json::from_str(&resp);
        match parsed {
            Ok(obj) => Ok(obj),
            Err(e) => Err(format!("Error parsing response: {}", e)),
        }
    }

    async fn read_commit(&self, id: &str) -> Result<Commit, String> {
        let request = ServerRequest::ReadCommit(ReadCommitRequest {
            repo_name: self.repo_name.clone(),
            commit_id: String::from(id),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;
        Commit::from_json(&resp)
    }

    async fn insert_commit(&self, commit: &Commit) -> Result<(), String> {
        let request = ServerRequest::InsertCommit(InsertCommitRequest {
            repo_name: self.repo_name.clone(),
            commit: commit.clone(),
        });
        let _resp = execute_request(&self.client, &self.url, &request).await?;
        Ok(())
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<(), String> {
        let request = ServerRequest::CommitToBranch(CommitToBranchRequest {
            repo_name: self.repo_name.clone(),
            commit: commit.clone(),
            branch: branch.clone(),
        });
        let _resp = execute_request(&self.client, &self.url, &request).await?;
        Ok(())
    }

    async fn commit_exists(&self, id: &str) -> Result<bool, String> {
        let request = ServerRequest::CommitExists(CommitExistsRequest {
            repo_name: self.repo_name.clone(),
            commit_id: String::from(id),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;
        let parsed: serde_json::Result<bool> = serde_json::from_str(&resp);
        match parsed {
            Ok(obj) => Ok(obj),
            Err(e) => Err(format!("Error parsing response: {}", e)),
        }
    }

    async fn read_tree(&self, hash: &str) -> Result<Tree, String> {
        let request = ServerRequest::ReadTree(ReadTreeRequest {
            repo_name: self.repo_name.clone(),
            tree_hash: String::from(hash),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;
        Tree::from_json(&resp)
    }

    async fn save_tree(&self, tree: &Tree, hash: &str) -> Result<(), String> {
        let request = ServerRequest::SaveTree(SaveTreeRequest {
            repo_name: self.repo_name.clone(),
            tree: tree.clone(),
            hash: String::from(hash),
        });
        let _resp = execute_request(&self.client, &self.url, &request).await?;
        Ok(())
    }

    async fn insert_lock(&self, lock: &Lock) -> Result<(), String> {
        let request = ServerRequest::InsertLock(InsertLockRequest {
            repo_name: self.repo_name.clone(),
            lock: lock.clone(),
        });
        let _resp = execute_request(&self.client, &self.url, &request).await?;
        Ok(())
    }

    async fn find_lock(
        &self,
        lock_domain_id: &str,
        canonical_relative_path: &str,
    ) -> Result<Option<Lock>, String> {
        let request = ServerRequest::FindLock(FindLockRequest {
            repo_name: self.repo_name.clone(),
            lock_domain_id: String::from(lock_domain_id),
            canonical_relative_path: String::from(canonical_relative_path),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;
        let parsed: serde_json::Result<Option<Lock>> = serde_json::from_str(&resp);
        match parsed {
            Ok(obj) => Ok(obj),
            Err(e) => Err(format!("Error parsing response: {}", e)),
        }
    }

    async fn find_locks_in_domain(&self, lock_domain_id: &str) -> Result<Vec<Lock>, String> {
        let request = ServerRequest::FindLocksInDomain(FindLocksInDomainRequest {
            repo_name: self.repo_name.clone(),
            lock_domain_id: String::from(lock_domain_id),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;
        let parsed: serde_json::Result<Vec<Lock>> = serde_json::from_str(&resp);
        match parsed {
            Ok(obj) => Ok(obj),
            Err(e) => Err(format!("Error parsing response: {}", e)),
        }
    }

    async fn clear_lock(
        &self,
        lock_domain_id: &str,
        canonical_relative_path: &str,
    ) -> Result<(), String> {
        let request = ServerRequest::ClearLock(ClearLockRequest {
            repo_name: self.repo_name.clone(),
            lock_domain_id: String::from(lock_domain_id),
            canonical_relative_path: String::from(canonical_relative_path),
        });
        let _resp = execute_request(&self.client, &self.url, &request).await?;
        Ok(())
    }

    async fn count_locks_in_domain(&self, lock_domain_id: &str) -> Result<i32, String> {
        let request = ServerRequest::ClountLocksInDomain(ClountLocksInDomainRequest {
            repo_name: self.repo_name.clone(),
            lock_domain_id: String::from(lock_domain_id),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;
        let parsed: serde_json::Result<i32> = serde_json::from_str(&resp);
        match parsed {
            Ok(obj) => Ok(obj),
            Err(e) => Err(format!("Error parsing response: {}", e)),
        }
    }

    async fn read_blob_storage_spec(&self) -> Result<BlobStorageSpec, String> {
        let request = ServerRequest::ReadBlobStorageSpec(ReadBlobStorageSpecRequest {
            repo_name: self.repo_name.clone(),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;
        Ok(BlobStorageSpec::from_json(&resp)?)
    }
}
