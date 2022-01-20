use async_trait::async_trait;
use lgn_online::grpc::GrpcWebClient;
use lgn_tracing::debug;
use tokio::sync::Mutex;
use url::Url;

use lgn_source_control_proto::{
    source_control_client::SourceControlClient, ClearLockRequest, CommitExistsRequest,
    CommitToBranchRequest, CountLocksInDomainRequest, CreateIndexRequest, DestroyIndexRequest,
    FindBranchRequest, FindBranchesInLockDomainRequest, FindLockRequest, FindLocksInDomainRequest,
    GetBlobStorageUrlRequest, IndexExistsRequest, InsertBranchRequest, InsertCommitRequest,
    InsertLockRequest, ReadBranchesRequest, ReadCommitRequest, ReadTreeRequest,
    RegisterWorkspaceRequest, SaveTreeRequest, UpdateBranchRequest,
};

use crate::{
    BlobStorageUrl, Branch, Commit, Error, IndexBackend, Lock, MapOtherError, Result, Tree,
    WorkspaceRegistration,
};

// Access to repository metadata through a gRPC server.
pub struct GrpcIndexBackend {
    url: Url,
    repository_name: String,
    client: Mutex<SourceControlClient<GrpcWebClient>>,
}

impl GrpcIndexBackend {
    pub fn new(url: Url) -> Result<Self> {
        let repository_name = url.path().trim_start_matches('/').to_string();
        let mut grpc_url = url.clone();
        grpc_url.set_path("");

        if repository_name.is_empty() {
            return Err(Error::invalid_index_url(
                url,
                anyhow::anyhow!("invalid empty repository name"),
            ));
        }

        debug!(
            "gRPC index backend instance targets repository `{}` at: {}",
            repository_name, grpc_url
        );

        // The `to_string` hereafter is hardly optimal but this should not live
        // in a critical path anyway so it probably does not matter.
        //
        // If the conversion bothers you, feel free to change it.
        let client = GrpcWebClient::new(grpc_url.to_string().parse().unwrap());
        let client = Mutex::new(SourceControlClient::new(client));

        Ok(Self {
            url,
            repository_name,
            client,
        })
    }
}

#[async_trait]
impl IndexBackend for GrpcIndexBackend {
    fn url(&self) -> &str {
        self.url.as_str()
    }

    async fn create_index(&self) -> Result<BlobStorageUrl> {
        let resp = self
            .client
            .lock()
            .await
            .create_index(CreateIndexRequest {
                repository_name: self.repository_name.clone(),
            })
            .await
            .map_other_err(format!("failed to create index `{}`", self.repository_name))?
            .into_inner();

        if resp.already_exists {
            return Err(Error::index_already_exists(self.url()));
        }

        resp.blob_storage_url
            .parse()
            .map_other_err("failed to parse the returned blob storage url")
    }

    async fn destroy_index(&self) -> Result<()> {
        let resp = self
            .client
            .lock()
            .await
            .destroy_index(DestroyIndexRequest {
                repository_name: self.repository_name.clone(),
            })
            .await
            .map_other_err(format!(
                "failed to destroy index `{}`",
                self.repository_name
            ))?
            .into_inner();

        if resp.does_not_exist {
            return Err(Error::index_does_not_exist(self.url()));
        }

        Ok(())
    }

    async fn index_exists(&self) -> Result<bool> {
        let resp = self
            .client
            .lock()
            .await
            .index_exists(IndexExistsRequest {
                repository_name: self.repository_name.clone(),
            })
            .await
            .map_other_err("failed to check if index exists")?
            .into_inner();

        Ok(resp.exists)
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

    async fn read_tree(&self, id: &str) -> Result<Tree> {
        let resp = self
            .client
            .lock()
            .await
            .read_tree(ReadTreeRequest {
                repository_name: self.repository_name.clone(),
                tree_id: id.into(),
            })
            .await
            .map_other_err(format!("failed to read tree `{}`", id))?
            .into_inner();

        Ok(resp.tree.unwrap_or_default().into())
    }

    async fn save_tree(&self, tree: &Tree) -> Result<String> {
        self.client
            .lock()
            .await
            .save_tree(SaveTreeRequest {
                repository_name: self.repository_name.clone(),
                tree: Some(tree.clone().into()),
            })
            .await
            .map_other_err("failed to save tree")
            .map(|resp| resp.into_inner().tree_id)
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
