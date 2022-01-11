use anyhow::Result;
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

use crate::{BlobStorageUrl, Branch, Commit, Lock, RepositoryQuery, Tree, WorkspaceRegistration};

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
        self.client.lock().await.ping(()).await?;

        Ok(())
    }

    async fn create_repository(
        &self,
        blob_storage_url: Option<BlobStorageUrl>,
    ) -> Result<BlobStorageUrl> {
        if blob_storage_url.is_some() {
            return Err(anyhow::anyhow!(
                "specfiying a blob storage url is not supported by this repository query"
            ));
        }

        let resp = self
            .client
            .lock()
            .await
            .create_repository(CreateRepositoryRequest {
                repository_name: self.repository_name.clone(),
            })
            .await?
            .into_inner();

        resp.blob_storage_url.parse()
    }

    async fn destroy_repository(&self) -> Result<()> {
        self.client
            .lock()
            .await
            .destroy_repository(DestroyRepositoryRequest {
                repository_name: self.repository_name.clone(),
            })
            .await?;

        Ok(())
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
            .await?;

        Ok(())
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
            .await?
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
            .await?
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
            .await?;

        Ok(())
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        self.client
            .lock()
            .await
            .update_branch(UpdateBranchRequest {
                repository_name: self.repository_name.clone(),
                branch: Some(branch.clone().into()),
            })
            .await?;

        Ok(())
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
            .await?
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
            .await?
            .into_inner();

        resp.commit.unwrap_or_default().try_into()
    }

    async fn insert_commit(&self, commit: &Commit) -> Result<()> {
        self.client
            .lock()
            .await
            .insert_commit(InsertCommitRequest {
                repository_name: self.repository_name.clone(),
                commit: Some(commit.clone().into()),
            })
            .await?;

        Ok(())
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
            .await?;

        Ok(())
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
            .await?
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
            .await?
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
            .await?;

        Ok(())
    }

    async fn insert_lock(&self, lock: &Lock) -> Result<()> {
        self.client
            .lock()
            .await
            .insert_lock(InsertLockRequest {
                repository_name: self.repository_name.clone(),
                lock: Some(lock.clone().into()),
            })
            .await?;

        Ok(())
    }

    async fn find_lock(
        &self,
        lock_domain_id: &str,
        canonical_relative_path: &str,
    ) -> Result<Option<Lock>> {
        let resp = self
            .client
            .lock()
            .await
            .find_lock(FindLockRequest {
                repository_name: self.repository_name.clone(),
                lock_domain_id: lock_domain_id.into(),
                canonical_relative_path: canonical_relative_path.into(),
            })
            .await?
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
            .await?
            .into_inner();

        Ok(resp.locks.into_iter().map(Into::into).collect())
    }

    async fn clear_lock(&self, lock_domain_id: &str, canonical_relative_path: &str) -> Result<()> {
        self.client
            .lock()
            .await
            .clear_lock(ClearLockRequest {
                repository_name: self.repository_name.clone(),
                lock_domain_id: lock_domain_id.into(),
                canonical_relative_path: canonical_relative_path.into(),
            })
            .await?;

        Ok(())
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
            .await?
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
            .await?
            .into_inner();

        resp.blob_storage_url.parse()
    }
}
