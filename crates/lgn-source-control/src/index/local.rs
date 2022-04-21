use std::path::Path;

use async_trait::async_trait;
use lgn_tracing::prelude::*;

use crate::{
    Branch, CanonicalPath, Commit, CommitId, Error, Index, ListBranchesQuery, ListCommitsQuery,
    ListLocksQuery, Lock, MapOtherError, RepositoryIndex, RepositoryName, Result,
    SqlRepositoryIndex, Tree, WorkspaceRegistration,
};

#[derive(Debug, Clone)]
pub struct LocalRepositoryIndex {
    inner_repository_index: SqlRepositoryIndex,
}

impl LocalRepositoryIndex {
    #[span_fn]
    pub async fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        if !root.is_absolute() {
            return Err(Error::Unspecified(format!(
                "expected absolute path, got: {}",
                root.display()
            )));
        }

        tokio::fs::create_dir_all(root)
            .await
            .map_other_err(format!("failed to create root at `{}`", root.display()))?;

        let db_path = root.join("repositories.db3");

        // Careful: here be dragons. You may be tempted to store the SQLite url
        // in a `Url` but this will break SQLite on Windows, as attempting to
        // parse a SQLite URL like:
        //
        // sqlite:///C:/Users/user/repo/repo.db3
        //
        // Will actually remove the trailing ':' from the disk letter.

        let sqlite_url = format!("sqlite://{}", db_path.to_str().unwrap().replace('\\', "/"));

        let inner_repository_index = SqlRepositoryIndex::new(sqlite_url).await?;

        Ok(Self {
            inner_repository_index,
        })
    }
}

#[async_trait]
impl RepositoryIndex for LocalRepositoryIndex {
    #[span_fn]
    async fn create_repository(&self, repository_name: RepositoryName) -> Result<Box<dyn Index>> {
        let inner_index = self
            .inner_repository_index
            .create_repository(repository_name)
            .await?;

        let index = LocalIndex::new(inner_index);

        Ok(Box::new(index))
    }

    #[span_fn]
    async fn destroy_repository(&self, repository_name: RepositoryName) -> Result<()> {
        self.inner_repository_index
            .destroy_repository(repository_name)
            .await
    }

    #[span_fn]
    async fn load_repository(&self, repository_name: RepositoryName) -> Result<Box<dyn Index>> {
        let inner_index = self
            .inner_repository_index
            .load_repository(repository_name)
            .await?;

        let index = LocalIndex::new(inner_index);

        Ok(Box::new(index))
    }

    #[span_fn]
    async fn list_repositories(&self) -> Result<Vec<RepositoryName>> {
        self.inner_repository_index.list_repositories().await
    }
}

pub struct LocalIndex {
    inner_index: Box<dyn Index>,
}

impl LocalIndex {
    fn new(inner_index: Box<dyn Index>) -> Self {
        Self { inner_index }
    }
}

#[async_trait]
impl Index for LocalIndex {
    fn repository_name(&self) -> &RepositoryName {
        self.inner_index.repository_name()
    }

    #[span_fn]
    async fn register_workspace(
        &self,
        workspace_registration: &WorkspaceRegistration,
    ) -> Result<()> {
        self.inner_index
            .register_workspace(workspace_registration)
            .await
    }

    #[span_fn]
    async fn get_branch(&self, branch_name: &str) -> Result<Branch> {
        self.inner_index.get_branch(branch_name).await
    }

    #[span_fn]
    async fn list_branches(&self, query: &ListBranchesQuery<'_>) -> Result<Vec<Branch>> {
        self.inner_index.list_branches(query).await
    }

    #[span_fn]
    async fn insert_branch(&self, branch: &Branch) -> Result<()> {
        self.inner_index.insert_branch(branch).await
    }

    #[span_fn]
    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        self.inner_index.update_branch(branch).await
    }

    #[span_fn]
    async fn list_commits(&self, query: &ListCommitsQuery) -> Result<Vec<Commit>> {
        self.inner_index.list_commits(query).await
    }

    #[span_fn]
    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<CommitId> {
        self.inner_index.commit_to_branch(commit, branch).await
    }

    #[span_fn]
    async fn get_tree(&self, id: &str) -> Result<Tree> {
        self.inner_index.get_tree(id).await
    }

    #[span_fn]
    async fn save_tree(&self, tree: &Tree) -> Result<String> {
        self.inner_index.save_tree(tree).await
    }

    #[span_fn]
    async fn lock(&self, lock: &Lock) -> Result<()> {
        self.inner_index.lock(lock).await
    }

    #[span_fn]
    async fn get_lock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<Lock> {
        self.inner_index
            .get_lock(lock_domain_id, canonical_path)
            .await
    }

    #[span_fn]
    async fn list_locks(&self, query: &ListLocksQuery<'_>) -> Result<Vec<Lock>> {
        self.inner_index.list_locks(query).await
    }

    #[span_fn]
    async fn unlock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<()> {
        self.inner_index
            .unlock(lock_domain_id, canonical_path)
            .await
    }

    #[span_fn]
    async fn count_locks(&self, query: &ListLocksQuery<'_>) -> Result<i32> {
        self.inner_index.count_locks(query).await
    }
}

/*#[cfg(test)]
mod tests {
    use crate::{IndexBackend, LocalIndexBackend};

    //#[tracing::instrument]
    async fn test() {
        println!("Hello world");

        let root = tempfile::tempdir().unwrap();
        {
            let mut index = LocalIndexBackend::new(root.path()).unwrap();
            index.create_index().await.unwrap();
            index.close().await;
            //}
            tokio::time::sleep(tokio::time::Duration::from_micros(1)).await;
            //{
            //let index = LocalIndexBackend::new(root.path()).unwrap();
            index.destroy_index().await.unwrap();
        }
    }

    #[test]
    fn create_destroy() {
        //tracing_subscriber::fmt::init();

        //console_subscriber::ConsoleLayer::builder()
        //    .with_default_env()
        //    .recording_path("D://recording.txt")
        //    .init();

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(test());
    }
}*/
