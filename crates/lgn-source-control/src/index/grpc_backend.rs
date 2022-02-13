use async_trait::async_trait;
use lgn_online::grpc::GrpcWebClient;
use lgn_tracing::debug;
use tokio::sync::Mutex;
use url::Url;

use lgn_source_control_proto::{
    source_control_client::SourceControlClient, CommitToBranchRequest, CountLocksRequest,
    CreateIndexRequest, DestroyIndexRequest, GetBlobStorageUrlRequest, GetBranchRequest,
    GetLockRequest, GetTreeRequest, IndexExistsRequest, InsertBranchRequest, ListBranchesRequest,
    ListCommitsRequest, ListLocksRequest, LockRequest, RegisterWorkspaceRequest, SaveTreeRequest,
    UnlockRequest, UpdateBranchRequest,
};

use crate::{
    BlobStorageUrl, Branch, CanonicalPath, Commit, CommitId, Error, IndexBackend,
    ListBranchesQuery, ListCommitsQuery, ListLocksQuery, Lock, MapOtherError, Result, Tree,
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

    async fn get_branch(&self, branch_name: &str) -> Result<Branch> {
        let resp = self
            .client
            .lock()
            .await
            .get_branch(GetBranchRequest {
                repository_name: self.repository_name.clone(),
                branch_name: branch_name.into(),
            })
            .await
            .map_other_err(format!("failed to find branch `{}`", branch_name))?
            .into_inner();

        match resp.branch {
            Some(branch) => Ok(branch.into()),
            None => Err(Error::branch_not_found(branch_name.to_string())),
        }
    }

    async fn list_branches(&self, query: &ListBranchesQuery<'_>) -> Result<Vec<Branch>> {
        let resp = self
            .client
            .lock()
            .await
            .list_branches(ListBranchesRequest {
                repository_name: self.repository_name.clone(),
                lock_domain_id: query.lock_domain_id.unwrap_or_default().into(),
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

    async fn list_commits(&self, query: &ListCommitsQuery) -> Result<Vec<Commit>> {
        let resp = self
            .client
            .lock()
            .await
            .list_commits(ListCommitsRequest {
                repository_name: self.repository_name.clone(),
                commit_ids: query
                    .commit_ids
                    .iter()
                    .map(|commit_id| commit_id.0)
                    .collect(),
                depth: query.depth,
            })
            .await
            .map_other_err("failed to list commits")?
            .into_inner();

        resp.commits
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()
            .map_other_err("failed to parse commits")
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<CommitId> {
        let resp = self
            .client
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
            ))?
            .into_inner();

        Ok(CommitId(resp.commit_id))
    }

    async fn get_tree(&self, id: &str) -> Result<Tree> {
        let resp = self
            .client
            .lock()
            .await
            .get_tree(GetTreeRequest {
                repository_name: self.repository_name.clone(),
                tree_id: id.into(),
            })
            .await
            .map_other_err(format!("failed to get tree `{}`", id))?
            .into_inner();

        resp.tree.unwrap_or_default().try_into()
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

    async fn lock(&self, lock: &Lock) -> Result<()> {
        self.client
            .lock()
            .await
            .lock(LockRequest {
                repository_name: self.repository_name.clone(),
                lock: Some(lock.clone().into()),
            })
            .await
            .map_other_err(format!(
                "failed to create lock `{}/{}`",
                lock.lock_domain_id, lock.canonical_path,
            ))
            .map(|_| ())
    }

    async fn unlock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<()> {
        self.client
            .lock()
            .await
            .unlock(UnlockRequest {
                repository_name: self.repository_name.clone(),
                lock_domain_id: lock_domain_id.into(),
                canonical_path: canonical_path.to_string(),
            })
            .await
            .map_other_err(format!(
                "failed to clear lock `{}/{}`",
                lock_domain_id, canonical_path,
            ))
            .map(|_| ())
    }

    async fn get_lock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<Lock> {
        let resp = self
            .client
            .lock()
            .await
            .get_lock(GetLockRequest {
                repository_name: self.repository_name.clone(),
                lock_domain_id: lock_domain_id.into(),
                canonical_path: canonical_path.to_string(),
            })
            .await
            .map_other_err(format!(
                "failed to find lock `{}` in lock domain `{}`",
                canonical_path, lock_domain_id
            ))?
            .into_inner();

        match resp.lock {
            Some(lock) => Ok(lock.try_into()?),
            None => Err(Error::lock_not_found(
                lock_domain_id.to_string(),
                canonical_path.clone(),
            )),
        }
    }

    async fn list_locks(&self, query: &ListLocksQuery<'_>) -> Result<Vec<Lock>> {
        let resp = self
            .client
            .lock()
            .await
            .list_locks(ListLocksRequest {
                repository_name: self.repository_name.clone(),
                lock_domain_ids: query
                    .lock_domain_ids
                    .iter()
                    .copied()
                    .map(Into::into)
                    .collect(),
            })
            .await
            .map_other_err("failed to list locks")?
            .into_inner();

        resp.locks.into_iter().map(TryInto::try_into).collect()
    }

    async fn count_locks(&self, query: &ListLocksQuery<'_>) -> Result<i32> {
        let resp = self
            .client
            .lock()
            .await
            .count_locks(CountLocksRequest {
                repository_name: self.repository_name.clone(),
                lock_domain_ids: query
                    .lock_domain_ids
                    .iter()
                    .copied()
                    .map(Into::into)
                    .collect(),
            })
            .await
            .map_other_err("failed to count locks")?
            .into_inner();

        Ok(resp.count)
    }
}
