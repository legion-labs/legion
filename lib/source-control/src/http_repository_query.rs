use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Url;

use crate::{
    execute_request, BlobStorageUrl, Branch, ClearLockRequest, ClountLocksInDomainRequest, Commit,
    CommitExistsRequest, CommitToBranchRequest, FindBranchRequest, FindBranchesInLockDomainRequest,
    FindLockRequest, FindLocksInDomainRequest, InitRepositoryRequest, InsertBranchRequest,
    InsertCommitRequest, InsertLockRequest, InsertWorkspaceRequest, Lock,
    ReadBlobStorageSpecRequest, ReadBranchesRequest, ReadCommitRequest, ReadTreeRequest,
    RepositoryQuery, SaveTreeRequest, ServerRequest, Tree, UpdateBranchRequest, Workspace,
};

// access to repository metadata through a web server
pub struct HttpRepositoryQuery {
    url: Url,
    repo_name: String,
    client: reqwest::Client,
}

impl HttpRepositoryQuery {
    pub fn new(url: Url, repo_name: String) -> Self {
        Self {
            url,
            repo_name,
            client: reqwest::Client::new(),
        }
    }

    pub async fn create_repository(&self, name: &str) -> Result<BlobStorageUrl> {
        let request = ServerRequest::InitRepo(InitRepositoryRequest {
            repo_name: String::from(name),
        });

        let resp = execute_request(&self.client, &self.url, &request).await?;

        resp.parse()
    }
}

#[async_trait]
impl RepositoryQuery for HttpRepositoryQuery {
    async fn insert_workspace(&self, spec: &Workspace) -> Result<()> {
        let request = ServerRequest::InsertWorkspace(InsertWorkspaceRequest {
            repo_name: self.repo_name.clone(),
            spec: spec.clone(),
        });

        execute_request(&self.client, &self.url, &request).await?;

        Ok(())
    }

    async fn read_branch(&self, name: &str) -> Result<Branch> {
        self.find_branch(name)
            .await
            .context("read_branch")?
            .ok_or_else(|| anyhow::format_err!("branch {} not found", name))
    }

    async fn find_branch(&self, name: &str) -> Result<Option<Branch>> {
        let request = ServerRequest::FindBranch(FindBranchRequest {
            repo_name: self.repo_name.clone(),
            branch_name: String::from(name),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;

        serde_json::from_str(&resp).context("parsing response")
    }

    async fn read_branches(&self) -> Result<Vec<Branch>> {
        let request = ServerRequest::ReadBranches(ReadBranchesRequest {
            repo_name: self.repo_name.clone(),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;

        serde_json::from_str(&resp).context("parsing response")
    }

    async fn insert_branch(&self, branch: &Branch) -> Result<()> {
        let request = ServerRequest::InsertBranch(InsertBranchRequest {
            repo_name: self.repo_name.clone(),
            branch: branch.clone(),
        });
        let _resp = execute_request(&self.client, &self.url, &request).await?;
        Ok(())
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        let request = ServerRequest::UpdateBranch(UpdateBranchRequest {
            repo_name: self.repo_name.clone(),
            branch: branch.clone(),
        });
        let _resp = execute_request(&self.client, &self.url, &request).await?;
        Ok(())
    }

    async fn find_branches_in_lock_domain(&self, lock_domain_id: &str) -> Result<Vec<Branch>> {
        let request = ServerRequest::FindBranchesInLockDomain(FindBranchesInLockDomainRequest {
            repo_name: self.repo_name.clone(),
            lock_domain_id: String::from(lock_domain_id),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;

        serde_json::from_str(&resp).context("parsing response")
    }

    async fn read_commit(&self, id: &str) -> Result<Commit> {
        let request = ServerRequest::ReadCommit(ReadCommitRequest {
            repo_name: self.repo_name.clone(),
            commit_id: String::from(id),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;
        Commit::from_json(&resp)
    }

    async fn insert_commit(&self, commit: &Commit) -> Result<()> {
        let request = ServerRequest::InsertCommit(InsertCommitRequest {
            repo_name: self.repo_name.clone(),
            commit: commit.clone(),
        });
        let _resp = execute_request(&self.client, &self.url, &request).await?;
        Ok(())
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<()> {
        let request = ServerRequest::CommitToBranch(CommitToBranchRequest {
            repo_name: self.repo_name.clone(),
            commit: commit.clone(),
            branch: branch.clone(),
        });
        let _resp = execute_request(&self.client, &self.url, &request).await?;
        Ok(())
    }

    async fn commit_exists(&self, id: &str) -> Result<bool> {
        let request = ServerRequest::CommitExists(CommitExistsRequest {
            repo_name: self.repo_name.clone(),
            commit_id: String::from(id),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;

        serde_json::from_str(&resp).context("parsing response")
    }

    async fn read_tree(&self, hash: &str) -> Result<Tree> {
        let request = ServerRequest::ReadTree(ReadTreeRequest {
            repo_name: self.repo_name.clone(),
            tree_hash: String::from(hash),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;
        Tree::from_json(&resp)
    }

    async fn save_tree(&self, tree: &Tree, hash: &str) -> Result<()> {
        let request = ServerRequest::SaveTree(SaveTreeRequest {
            repo_name: self.repo_name.clone(),
            tree: tree.clone(),
            hash: String::from(hash),
        });
        let _resp = execute_request(&self.client, &self.url, &request).await?;
        Ok(())
    }

    async fn insert_lock(&self, lock: &Lock) -> Result<()> {
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
    ) -> Result<Option<Lock>> {
        let request = ServerRequest::FindLock(FindLockRequest {
            repo_name: self.repo_name.clone(),
            lock_domain_id: String::from(lock_domain_id),
            canonical_relative_path: String::from(canonical_relative_path),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;

        serde_json::from_str(&resp).context("parsing response")
    }

    async fn find_locks_in_domain(&self, lock_domain_id: &str) -> Result<Vec<Lock>> {
        let request = ServerRequest::FindLocksInDomain(FindLocksInDomainRequest {
            repo_name: self.repo_name.clone(),
            lock_domain_id: String::from(lock_domain_id),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;

        serde_json::from_str(&resp).context("parsing response")
    }

    async fn clear_lock(&self, lock_domain_id: &str, canonical_relative_path: &str) -> Result<()> {
        let request = ServerRequest::ClearLock(ClearLockRequest {
            repo_name: self.repo_name.clone(),
            lock_domain_id: String::from(lock_domain_id),
            canonical_relative_path: String::from(canonical_relative_path),
        });
        let _resp = execute_request(&self.client, &self.url, &request).await?;
        Ok(())
    }

    async fn count_locks_in_domain(&self, lock_domain_id: &str) -> Result<i32> {
        let request = ServerRequest::ClountLocksInDomain(ClountLocksInDomainRequest {
            repo_name: self.repo_name.clone(),
            lock_domain_id: String::from(lock_domain_id),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;

        serde_json::from_str(&resp).context("parsing response")
    }

    async fn read_blob_storage_spec(&self) -> Result<BlobStorageUrl> {
        let request = ServerRequest::ReadBlobStorageSpec(ReadBlobStorageSpecRequest {
            repo_name: self.repo_name.clone(),
        });
        let resp = execute_request(&self.client, &self.url, &request).await?;

        serde_json::from_str(&resp).context("parsing response")
    }
}
