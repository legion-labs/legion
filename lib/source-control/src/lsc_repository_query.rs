use async_trait::async_trait;
use lgn_online::grpc::GrpcWebClient;
use lgn_tracing::debug;
use tokio::sync::Mutex;
use url::Url;

use lgn_source_control_proto::{
    source_control_client::SourceControlClient, ClearLockRequest, CommitExistsRequest,
    CommitToBranchRequest, CountLocksInDomainRequest, CreateRepositoryRequest,
    DestroyRepositoryRequest, FindBranchRequest, FindBranchesInLockDomainRequest, FindLockRequest,
    FindLocksInDomainRequest, GetBlobStorageUrlRequest, InsertBranchRequest, InsertCommitRequest,
    InsertLockRequest, ReadBranchesRequest, ReadCommitRequest, ReadTreeRequest,
    RegisterWorkspaceRequest, SaveTreeRequest, UpdateBranchRequest,
};

use crate::{
    blob_storage::BlobStorageUrl, Branch, Commit, Error, Lock, MapOtherError, RepositoryQuery,
    Result, Tree, WorkspaceRegistration,
};

// Access to repository metadata through a gRPC server.
pub struct LscRepositoryQuery {
    repository_name: String,
    client: Mutex<SourceControlClient<GrpcWebClient>>,
}

impl LscRepositoryQuery {
    pub fn new(mut url: Url) -> Self {
        let repository_name = url.path().trim_start_matches('/').to_string();
        url.set_path("");

        debug!(
            "Instance targets repository `{}` at: {}",
            repository_name, url
        );

        // TODO: To `to_string` hereafter is hardly optimal but this should not
        // live in a critical path anyway so it probably does not matter.
        //
        // If the conversion bothers you, feel free to change it.
        let client = GrpcWebClient::new(url.to_string().parse().unwrap());
        let client = Mutex::new(SourceControlClient::new(client));

        Self {
            repository_name,
            client,
        }
    }
}

#[async_trait]
impl RepositoryQuery for LscRepositoryQuery {
    async fn ping(&self) -> Result<()> {
        self.client
            .lock()
            .await
            .ping(())
            .await
            .map_other_err("failed to ping repository")
            .map(|_| ())
    }

    async fn create_repository(
        &self,
        blob_storage_url: Option<BlobStorageUrl>,
    ) -> Result<BlobStorageUrl> {
        if let Some(blob_storage_url) = blob_storage_url {
            return Err(Error::unexpected_blob_storage_url(blob_storage_url));
        }

        let resp = self
            .client
            .lock()
            .await
            .create_repository(CreateRepositoryRequest {
                repository_name: self.repository_name.clone(),
            })
            .await
            .map_other_err(format!(
                "failed to create repository `{}`",
                self.repository_name
            ))?
            .into_inner();

        resp.blob_storage_url
            .parse()
            .map_other_err("failed to parse the returned blob storage url")
    }

    async fn destroy_repository(&self) -> Result<()> {
        self.client
            .lock()
            .await
            .destroy_repository(DestroyRepositoryRequest {
                repository_name: self.repository_name.clone(),
            })
            .await
            .map_other_err(format!(
                "failed to destroy repository `{}`",
                self.repository_name
            ))
            .map(|_| ())
    }

    async fn register_workspace(
        &self,
        workspace_registration: &WorkspaceRegistration,
    ) -> Result<()> {
        self.client
            .lock()
            .await
            .register_workspace(RegisterWorkspaceRequest {
                repository_name: self.repository_name.clone(),
                workspace_registration: Some(workspace_registration.clone().into()),
            })
            .await
            .map_other_err(format!(
                "failed to register workspace `{}`",
                workspace_registration.id
            ))
            .map(|_| ())
    }

    async fn find_branch(&self, branch_name: &str) -> Result<Option<Branch>> {
        let resp = self
            .client
            .lock()
            .await
            .find_branch(FindBranchRequest {
                repository_name: self.repository_name.clone(),
                branch_name: branch_name.into(),
            })
            .await
            .map_other_err(format!("failed to find branch `{}`", branch_name))?
            .into_inner();

        Ok(resp.branch.map(Into::into))
    }

    async fn read_branches(&self) -> Result<Vec<Branch>> {
        let resp = self
            .client
            .lock()
            .await
            .read_branches(ReadBranchesRequest {
                repository_name: self.repository_name.clone(),
            })
            .await
            .map_other_err("failed to read branches")?
            .into_inner();

        Ok(resp.branches.into_iter().map(Into::into).collect())
    }

    async fn insert_branch(&self, branch: &Branch) -> Result<()> {
        self.client
            .lock()
            .await
            .insert_branch(InsertBranchRequest {
                repository_name: self.repository_name.clone(),
                branch: Some(branch.clone().into()),
            })
            .await
            .map_other_err(format!("failed to insert branch `{}`", branch.name))
            .map(|_| ())
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        self.client
            .lock()
            .await
            .update_branch(UpdateBranchRequest {
                repository_name: self.repository_name.clone(),
                branch: Some(branch.clone().into()),
            })
            .await
            .map_other_err(format!("failed to update branch `{}`", branch.name))
            .map(|_| ())
    }

    async fn find_branches_in_lock_domain(&self, lock_domain_id: &str) -> Result<Vec<Branch>> {
        let resp = self
            .client
            .lock()
            .await
            .find_branches_in_lock_domain(FindBranchesInLockDomainRequest {
                repository_name: self.repository_name.clone(),
                lock_domain_id: lock_domain_id.into(),
            })
            .await
            .map_other_err(format!(
                "failed to find branches in lock domain `{}`",
                lock_domain_id
            ))?
            .into_inner();

        Ok(resp.branches.into_iter().map(Into::into).collect())
    }

    async fn read_commit(&self, commit_id: &str) -> Result<Commit> {
        let resp = self
            .client
            .lock()
            .await
            .read_commit(ReadCommitRequest {
                repository_name: self.repository_name.clone(),
                commit_id: commit_id.into(),
            })
            .await
            .map_other_err(format!("failed to read commit `{}`", commit_id))?
            .into_inner();

        resp.commit
            .unwrap_or_default()
            .try_into()
            .map_other_err("failed to parse commit")
    }

    async fn insert_commit(&self, commit: &Commit) -> Result<()> {
        self.client
            .lock()
            .await
            .insert_commit(InsertCommitRequest {
                repository_name: self.repository_name.clone(),
                commit: Some(commit.clone().into()),
            })
            .await
            .map_other_err(format!("failed to insert commit `{}`", commit.id))
            .map(|_| ())
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<()> {
        self.client
            .lock()
            .await
            .commit_to_branch(CommitToBranchRequest {
                repository_name: self.repository_name.clone(),
                commit: Some(commit.clone().into()),
                branch: Some(branch.clone().into()),
            })
            .await
            .map_other_err(format!(
                "failed to commit `{}` to branch `{}`",
                commit.id, branch.name
            ))
            .map(|_| ())
    }

    async fn commit_exists(&self, commit_id: &str) -> Result<bool> {
        let resp = self
            .client
            .lock()
            .await
            .commit_exists(CommitExistsRequest {
                repository_name: self.repository_name.clone(),
                commit_id: commit_id.into(),
            })
            .await
            .map_other_err(format!("failed to check if commit `{}` exists", commit_id))?
            .into_inner();

        Ok(resp.exists)
    }

    async fn read_tree(&self, tree_hash: &str) -> Result<Tree> {
        let resp = self
            .client
            .lock()
            .await
            .read_tree(ReadTreeRequest {
                repository_name: self.repository_name.clone(),
                tree_hash: tree_hash.into(),
            })
            .await
            .map_other_err(format!("failed to read tree `{}`", tree_hash))?
            .into_inner();

        Ok(resp.tree.unwrap_or_default().into())
    }

    async fn save_tree(&self, tree: &Tree, hash: &str) -> Result<()> {
        self.client
            .lock()
            .await
            .save_tree(SaveTreeRequest {
                repository_name: self.repository_name.clone(),
                tree: Some(tree.clone().into()),
                hash: hash.into(),
            })
            .await
            .map_other_err(format!("failed to save tree `{}`", hash))
            .map(|_| ())
    }

    async fn insert_lock(&self, lock: &Lock) -> Result<()> {
        self.client
            .lock()
            .await
            .insert_lock(InsertLockRequest {
                repository_name: self.repository_name.clone(),
                lock: Some(lock.clone().into()),
            })
            .await
            .map_other_err(format!(
                "failed to insert lock `{}` in domain `{}`",
                lock.relative_path, lock.lock_domain_id
            ))
            .map(|_| ())
    }

    async fn find_lock(&self, lock_domain_id: &str, relative_path: &str) -> Result<Option<Lock>> {
        let resp = self
            .client
            .lock()
            .await
            .find_lock(FindLockRequest {
                repository_name: self.repository_name.clone(),
                lock_domain_id: lock_domain_id.into(),
                canonical_relative_path: relative_path.into(),
            })
            .await
            .map_other_err(format!(
                "failed to find lock `{}` in lock domain `{}`",
                relative_path, lock_domain_id
            ))?
            .into_inner();

        Ok(resp.lock.map(Into::into))
    }

    async fn find_locks_in_domain(&self, lock_domain_id: &str) -> Result<Vec<Lock>> {
        let resp = self
            .client
            .lock()
            .await
            .find_locks_in_domain(FindLocksInDomainRequest {
                repository_name: self.repository_name.clone(),
                lock_domain_id: lock_domain_id.into(),
            })
            .await
            .map_other_err(format!(
                "failed to find locks in lock domain `{}`",
                lock_domain_id
            ))?
            .into_inner();

        Ok(resp.locks.into_iter().map(Into::into).collect())
    }

    async fn clear_lock(&self, lock_domain_id: &str, relative_path: &str) -> Result<()> {
        self.client
            .lock()
            .await
            .clear_lock(ClearLockRequest {
                repository_name: self.repository_name.clone(),
                lock_domain_id: lock_domain_id.into(),
                canonical_relative_path: relative_path.into(),
            })
            .await
            .map_other_err(format!(
                "failed to clear lock `{}` in lock domain `{}`",
                relative_path, lock_domain_id
            ))
            .map(|_| ())
    }

    async fn count_locks_in_domain(&self, lock_domain_id: &str) -> Result<i32> {
        let resp = self
            .client
            .lock()
            .await
            .count_locks_in_domain(CountLocksInDomainRequest {
                repository_name: self.repository_name.clone(),
                lock_domain_id: lock_domain_id.into(),
            })
            .await
            .map_other_err(format!(
                "failed to count locks in lock domain `{}`",
                lock_domain_id
            ))?
            .into_inner();

        Ok(resp.count)
    }

    async fn get_blob_storage_url(&self) -> Result<BlobStorageUrl> {
        let resp = self
            .client
            .lock()
            .await
            .get_blob_storage_url(GetBlobStorageUrlRequest {
                repository_name: self.repository_name.clone(),
            })
            .await
            .map_other_err("failed to get blob storage url")?
            .into_inner();

        resp.blob_storage_url
            .parse()
            .map_other_err("failed to parse blob storage url")
    }
}
