use crate::*;
use async_trait::async_trait;

// access to repository metadata through a web server
pub struct HTTPRepositoryQuery {
    url: String,
    repo_name: String,
}

impl HTTPRepositoryQuery {
    pub fn new(url: String, repo_name: String) -> Result<Self, String> {
        Ok(Self { url, repo_name })
    }
}

#[async_trait]
impl RepositoryQuery for HTTPRepositoryQuery {
    async fn insert_workspace(&self, spec: &Workspace) -> Result<(), String> {
        let request = ServerRequest::InsertWorkspace(InsertWorkspaceRequest {
            repo_name: self.repo_name.clone(),
            spec: spec.clone(),
        });
        let _resp = execute_request(&self.url, &request).await?;
        Ok(())
    }

    async fn read_branch(&self, name: &str) -> Result<Branch, String> {
        let request = ServerRequest::ReadBranch(ReadBranchRequest {
            repo_name: self.repo_name.clone(),
            branch_name: String::from(name),
        });
        let resp = execute_request(&self.url, &request).await?;
        Branch::from_json(&resp)
    }
    async fn insert_branch(&self, _branch: &Branch) -> Result<(), String> {
        panic!("not implemented");
    }
    async fn update_branch(&self, _branch: &Branch) -> Result<(), String> {
        panic!("not implemented");
    }
    async fn find_branch(&self, _name: &str) -> Result<Option<Branch>, String> {
        panic!("not implemented");
    }
    async fn find_branches_in_lock_domain(
        &self,
        _lock_domain_id: &str,
    ) -> Result<Vec<Branch>, String> {
        panic!("not implemented");
    }
    async fn read_branches(&self) -> Result<Vec<Branch>, String> {
        panic!("not implemented");
    }
    async fn read_commit(&self, id: &str) -> Result<Commit, String> {
        let request = ServerRequest::ReadCommit(ReadCommitRequest {
            repo_name: self.repo_name.clone(),
            commit_id: String::from(id),
        });
        let resp = execute_request(&self.url, &request).await?;
        Commit::from_json(&resp)
    }
    async fn insert_commit(&self, _commit: &Commit) -> Result<(), String> {
        panic!("not implemented");
    }
    async fn commit_exists(&self, _id: &str) -> Result<bool, String> {
        panic!("not implemented");
    }
    async fn read_tree(&self, _hash: &str) -> Result<Tree, String> {
        panic!("not implemented");
    }
    async fn save_tree(&self, _tree: &Tree, _hash: &str) -> Result<(), String> {
        panic!("not implemented");
    }
    async fn insert_lock(&self, _lock: &Lock) -> Result<(), String> {
        panic!("not implemented");
    }

    async fn find_lock(
        &self,
        _lock_domain_id: &str,
        _canonical_relative_path: &str,
    ) -> Result<Option<Lock>, String> {
        panic!("not implemented");
    }
    async fn find_locks_in_domain(&self, _lock_domain_id: &str) -> Result<Vec<Lock>, String> {
        panic!("not implemented");
    }
    async fn clear_lock(
        &self,
        _lock_domain_id: &str,
        _canonical_relative_path: &str,
    ) -> Result<(), String> {
        panic!("not implemented");
    }
    async fn count_locks_in_domain(&self, _lock_domain_id: &str) -> Result<i32, String> {
        panic!("not implemented");
    }
    async fn read_blob_storage_spec(&self) -> Result<BlobStorageSpec, String> {
        let request = ServerRequest::ReadBlobStorageSpec(ReadBlobStorageSpecRequest {
            repo_name: self.repo_name.clone(),
        });
        let resp = execute_request(&self.url, &request).await?;
        Ok(BlobStorageSpec::from_json(&resp)?)
    }
}
