use async_trait::async_trait;
use lgn_tracing::info;
use reqwest::Url;
use std::path::PathBuf;

use crate::{
    blob_storage::BlobStorageUrl, utils::make_path_absolute, Branch, Commit, Error, IndexBackend,
    Lock, MapOtherError, Result, SqlIndexBackend, Tree, WorkspaceRegistration,
};

pub struct LocalIndexBackend {
    directory: PathBuf,
    sql_repository_query: SqlIndexBackend,
}

impl LocalIndexBackend {
    pub fn new(directory: PathBuf) -> Result<Self> {
        let directory = make_path_absolute(directory);
        let db_path = directory.join("repo.db3");
        let blob_storage_url = &BlobStorageUrl::Local(directory.join("blobs"));
        let sqlite_url = Url::parse_with_params(
            &format!("sqlite://{}", db_path.to_str().unwrap().replace("\\", "/")),
            vec![("blob_storage_url", blob_storage_url.to_string())],
        )
        .map_other_err("failed to parse local SQLite repository database URL")?;

        Ok(Self {
            directory,
            sql_repository_query: SqlIndexBackend::new(&sqlite_url)?,
        })
    }
}

#[async_trait]
impl IndexBackend for LocalIndexBackend {
    fn url(&self) -> &str {
        self.directory.to_str().unwrap()
    }

    async fn create_index(&self) -> Result<BlobStorageUrl> {
        match self.directory.read_dir() {
            Ok(mut entries) => {
                if entries.next().is_some() {
                    return Err(Error::directory_already_exists(self.directory.clone()));
                }
            }
            Err(err) => {
                if err.kind() != std::io::ErrorKind::NotFound {
                    return Err(Error::Other {
                        source: err.into(),
                        context: format!(
                            "failed to read directory `{}`",
                            &self.directory.display()
                        ),
                    });
                }
            }
        };

        info!("Creating index root at {}", self.directory.display());

        tokio::fs::create_dir_all(&self.directory)
            .await
            .map_other_err(format!(
                "failed to create repository root at `{}`",
                &self.directory.display()
            ))?;

        info!("Creating SQLite database");

        self.sql_repository_query.create_index().await
    }

    async fn destroy_index(&self) -> Result<()> {
        self.sql_repository_query.destroy_index().await?;

        tokio::fs::remove_dir_all(&self.directory)
            .await
            .map_other_err(format!(
                "failed to destroy repository root at `{}`",
                &self.directory.display()
            ))
    }

    async fn index_exists(&self) -> Result<bool> {
        self.sql_repository_query.index_exists().await
    }

    async fn register_workspace(
        &self,
        workspace_registration: &WorkspaceRegistration,
    ) -> Result<()> {
        self.sql_repository_query
            .register_workspace(workspace_registration)
            .await
    }

    async fn find_branch(&self, branch_name: &str) -> Result<Option<Branch>> {
        self.sql_repository_query.find_branch(branch_name).await
    }

    async fn read_branches(&self) -> Result<Vec<Branch>> {
        self.sql_repository_query.read_branches().await
    }

    async fn insert_branch(&self, branch: &Branch) -> Result<()> {
        self.sql_repository_query.insert_branch(branch).await
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        self.sql_repository_query.update_branch(branch).await
    }

    async fn find_branches_in_lock_domain(&self, lock_domain_id: &str) -> Result<Vec<Branch>> {
        self.sql_repository_query
            .find_branches_in_lock_domain(lock_domain_id)
            .await
    }

    async fn read_commit(&self, commit_id: &str) -> Result<Commit> {
        self.sql_repository_query.read_commit(commit_id).await
    }

    async fn insert_commit(&self, commit: &Commit) -> Result<()> {
        self.sql_repository_query.insert_commit(commit).await
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<()> {
        self.sql_repository_query
            .commit_to_branch(commit, branch)
            .await
    }

    async fn commit_exists(&self, commit_id: &str) -> Result<bool> {
        self.sql_repository_query.commit_exists(commit_id).await
    }

    async fn read_tree(&self, tree_hash: &str) -> Result<Tree> {
        self.sql_repository_query.read_tree(tree_hash).await
    }

    async fn save_tree(&self, tree: &Tree, hash: &str) -> Result<()> {
        self.sql_repository_query.save_tree(tree, hash).await
    }

    async fn insert_lock(&self, lock: &Lock) -> Result<()> {
        self.sql_repository_query.insert_lock(lock).await
    }

    async fn find_lock(
        &self,
        lock_domain_id: &str,
        canonical_relative_path: &str,
    ) -> Result<Option<Lock>> {
        self.sql_repository_query
            .find_lock(lock_domain_id, canonical_relative_path)
            .await
    }

    async fn find_locks_in_domain(&self, lock_domain_id: &str) -> Result<Vec<Lock>> {
        self.sql_repository_query
            .find_locks_in_domain(lock_domain_id)
            .await
    }

    async fn clear_lock(&self, lock_domain_id: &str, canonical_relative_path: &str) -> Result<()> {
        self.sql_repository_query
            .clear_lock(lock_domain_id, canonical_relative_path)
            .await
    }

    async fn count_locks_in_domain(&self, lock_domain_id: &str) -> Result<i32> {
        self.sql_repository_query
            .count_locks_in_domain(lock_domain_id)
            .await
    }

    async fn get_blob_storage_url(&self) -> Result<BlobStorageUrl> {
        self.sql_repository_query.get_blob_storage_url().await
    }
}
