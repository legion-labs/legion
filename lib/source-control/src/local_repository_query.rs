use anyhow::{Context, Result};
use async_trait::async_trait;
use lgn_tracing::info;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    sql_repository_query::{DatabaseUri, SqlRepositoryQuery},
    utils::{check_directory_does_not_exist_or_is_empty, make_path_absolute},
    BlobStorageUrl, Branch, Commit, Lock, RepositoryQuery, Tree, WorkspaceRegistration,
};

pub struct LocalRepositoryQuery {
    directory: PathBuf,
    sqlite_url: String,
    sql_repository_query: Mutex<Option<Arc<SqlRepositoryQuery>>>,
}

impl LocalRepositoryQuery {
    pub fn new(directory: PathBuf) -> Self {
        let directory = make_path_absolute(directory);
        let db_path = directory.join("repo.db3");
        let sqlite_url = format!("sqlite://{}", db_path.to_str().unwrap().replace("\\", "/"));

        Self {
            directory,
            sqlite_url,
            sql_repository_query: Mutex::new(None),
        }
    }

    async fn sql_repository_query(&self) -> Result<Arc<SqlRepositoryQuery>> {
        let mut sql_repository_query = self.sql_repository_query.lock().await;

        if let Some(sql_repository_query) = sql_repository_query.as_ref() {
            Ok(Arc::clone(sql_repository_query))
        } else {
            let new_sql_repository_query = Arc::new(SqlRepositoryQuery::new(DatabaseUri::Sqlite(
                self.sqlite_url.clone(),
            )));

            *sql_repository_query = Some(Arc::clone(&new_sql_repository_query));

            Ok(new_sql_repository_query)
        }
    }
}

#[async_trait]
impl RepositoryQuery for LocalRepositoryQuery {
    async fn ping(&self) -> Result<()> {
        self.sql_repository_query().await?.ping().await
    }

    async fn create_repository(
        &self,
        blob_storage_url: Option<BlobStorageUrl>,
    ) -> Result<BlobStorageUrl> {
        check_directory_does_not_exist_or_is_empty(&self.directory)?;

        info!("Creating repository root at {}", self.directory.display());

        tokio::fs::create_dir_all(&self.directory)
            .await
            .context("could not create repository directory")?;

        info!("Creating SQLite database at {}", &self.sqlite_url);

        let default_blob_storage_url = &BlobStorageUrl::Local(self.directory.join("blobs"));
        let blob_storage_url = blob_storage_url
            .as_ref()
            .unwrap_or(default_blob_storage_url);

        self.sql_repository_query()
            .await?
            .create_repository(Some(blob_storage_url.clone()))
            .await
    }

    async fn destroy_repository(&self) -> Result<()> {
        self.sql_repository_query()
            .await?
            .destroy_repository()
            .await?;

        tokio::fs::remove_dir_all(&self.directory)
            .await
            .map_err(Into::into)
    }

    async fn register_workspace(
        &self,
        workspace_registration: &WorkspaceRegistration,
    ) -> Result<()> {
        self.sql_repository_query()
            .await?
            .register_workspace(workspace_registration)
            .await
    }

    async fn find_branch(&self, branch_name: &str) -> Result<Option<Branch>> {
        self.sql_repository_query()
            .await?
            .find_branch(branch_name)
            .await
    }

    async fn read_branches(&self) -> Result<Vec<Branch>> {
        self.sql_repository_query().await?.read_branches().await
    }

    async fn insert_branch(&self, branch: &Branch) -> Result<()> {
        self.sql_repository_query()
            .await?
            .insert_branch(branch)
            .await
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        self.sql_repository_query()
            .await?
            .update_branch(branch)
            .await
    }

    async fn find_branches_in_lock_domain(&self, lock_domain_id: &str) -> Result<Vec<Branch>> {
        self.sql_repository_query()
            .await?
            .find_branches_in_lock_domain(lock_domain_id)
            .await
    }

    async fn read_commit(&self, commit_id: &str) -> Result<Commit> {
        self.sql_repository_query()
            .await?
            .read_commit(commit_id)
            .await
    }

    async fn insert_commit(&self, commit: &Commit) -> Result<()> {
        self.sql_repository_query()
            .await?
            .insert_commit(commit)
            .await
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<()> {
        self.sql_repository_query()
            .await?
            .commit_to_branch(commit, branch)
            .await
    }

    async fn commit_exists(&self, commit_id: &str) -> Result<bool> {
        self.sql_repository_query()
            .await?
            .commit_exists(commit_id)
            .await
    }

    async fn read_tree(&self, tree_hash: &str) -> Result<Tree> {
        self.sql_repository_query()
            .await?
            .read_tree(tree_hash)
            .await
    }

    async fn save_tree(&self, tree: &Tree, hash: &str) -> Result<()> {
        self.sql_repository_query()
            .await?
            .save_tree(tree, hash)
            .await
    }

    async fn insert_lock(&self, lock: &Lock) -> Result<()> {
        self.sql_repository_query().await?.insert_lock(lock).await
    }

    async fn find_lock(
        &self,
        lock_domain_id: &str,
        canonical_relative_path: &str,
    ) -> Result<Option<Lock>> {
        self.sql_repository_query()
            .await?
            .find_lock(lock_domain_id, canonical_relative_path)
            .await
    }

    async fn find_locks_in_domain(&self, lock_domain_id: &str) -> Result<Vec<Lock>> {
        self.sql_repository_query()
            .await?
            .find_locks_in_domain(lock_domain_id)
            .await
    }

    async fn clear_lock(&self, lock_domain_id: &str, canonical_relative_path: &str) -> Result<()> {
        self.sql_repository_query()
            .await?
            .clear_lock(lock_domain_id, canonical_relative_path)
            .await
    }

    async fn count_locks_in_domain(&self, lock_domain_id: &str) -> Result<i32> {
        self.sql_repository_query()
            .await?
            .count_locks_in_domain(lock_domain_id)
            .await
    }

    async fn get_blob_storage_url(&self) -> Result<BlobStorageUrl> {
        self.sql_repository_query()
            .await?
            .get_blob_storage_url()
            .await
    }
}
