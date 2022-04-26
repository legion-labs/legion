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
    async fn create_repository(&self, repository_name: RepositoryName) -> Result<Box<dyn Index>> {
        async_span_scope!("LocalRepositoryIndex::create_repository");

        let inner_index = self
            .inner_repository_index
            .create_repository(repository_name)
            .await?;

        let index = LocalIndex::new(inner_index);

        Ok(Box::new(index))
    }

    async fn destroy_repository(&self, repository_name: RepositoryName) -> Result<()> {
        async_span_scope!("LocalRepositoryIndex::destroy_repository");

        self.inner_repository_index
            .destroy_repository(repository_name)
            .await
    }

    async fn load_repository(&self, repository_name: RepositoryName) -> Result<Box<dyn Index>> {
        async_span_scope!("LocalRepositoryIndex::load_repository");

        let inner_index = self
            .inner_repository_index
            .load_repository(repository_name)
            .await?;

        let index = LocalIndex::new(inner_index);

        Ok(Box::new(index))
    }

    async fn list_repositories(&self) -> Result<Vec<RepositoryName>> {
        async_span_scope!("LocalRepositoryIndex::list_repositories");

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

    async fn register_workspace(
        &self,
        workspace_registration: &WorkspaceRegistration,
    ) -> Result<()> {
        async_span_scope!("LocalIndexBackend::register_workspace");
        self.inner_index
            .register_workspace(workspace_registration)
            .await
    }

    async fn get_branch(&self, branch_name: &str) -> Result<Branch> {
        async_span_scope!("LocalIndexBackend::get_branch");
        self.inner_index.get_branch(branch_name).await
    }

    async fn list_branches(&self, query: &ListBranchesQuery<'_>) -> Result<Vec<Branch>> {
        async_span_scope!("LocalIndexBackend::list_branches");
        self.inner_index.list_branches(query).await
    }

    async fn insert_branch(&self, branch: &Branch) -> Result<()> {
        async_span_scope!("LocalIndexBackend::insert_branch");
        self.inner_index.insert_branch(branch).await
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        async_span_scope!("LocalIndexBackend::update_branch");
        self.inner_index.update_branch(branch).await
    }

    async fn list_commits(&self, query: &ListCommitsQuery) -> Result<Vec<Commit>> {
        async_span_scope!("LocalIndexBackend::list_commits");
        self.inner_index.list_commits(query).await
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<CommitId> {
        async_span_scope!("LocalIndexBackend::commit_to_branch");
        self.inner_index.commit_to_branch(commit, branch).await
    }

    async fn get_tree(&self, id: &str) -> Result<Tree> {
        async_span_scope!("LocalIndexBackend::get_tree");
        self.inner_index.get_tree(id).await
    }

    async fn save_tree(&self, tree: &Tree) -> Result<String> {
        async_span_scope!("LocalIndexBackend::save_tree");
        self.inner_index.save_tree(tree).await
    }

    async fn lock(&self, lock: &Lock) -> Result<()> {
        async_span_scope!("LocalIndexBackend::lock");
        self.inner_index.lock(lock).await
    }

    async fn get_lock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<Lock> {
        async_span_scope!("LocalIndexBackend::get_lock");
        self.inner_index
            .get_lock(lock_domain_id, canonical_path)
            .await
    }

    async fn list_locks(&self, query: &ListLocksQuery<'_>) -> Result<Vec<Lock>> {
        async_span_scope!("LocalIndexBackend::list_locks");
        self.inner_index.list_locks(query).await
    }

    async fn unlock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<()> {
        async_span_scope!("LocalIndexBackend::unlock");
        self.inner_index
            .unlock(lock_domain_id, canonical_path)
            .await
    }

    async fn count_locks(&self, query: &ListLocksQuery<'_>) -> Result<i32> {
        async_span_scope!("LocalIndexBackend::count_locks");
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
