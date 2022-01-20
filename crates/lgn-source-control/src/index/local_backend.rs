use async_trait::async_trait;
use lgn_tracing::info;
use std::path::{Path, PathBuf};

use crate::{
    BlobStorageUrl, Branch, Commit, Error, IndexBackend, Lock, MapOtherError, Result,
    SqlIndexBackend, Tree, WorkspaceRegistration,
};

#[derive(Debug)]
pub struct LocalIndexBackend {
    directory: PathBuf,
    sql_repository_index: SqlIndexBackend,
}

impl LocalIndexBackend {
    pub fn new(directory: impl AsRef<Path>) -> Result<Self> {
        if !directory.as_ref().is_absolute() {
            return Err(Error::invalid_index_url(
                directory.as_ref().to_str().unwrap(),
                anyhow::anyhow!("expected absolute directory"),
            ));
        }
        let db_path = directory.as_ref().join("repo.db3");
        let blob_storage_url = &BlobStorageUrl::Local(directory.as_ref().join("blobs"));

        // Careful: here be dragons. You may be tempted to store the SQLite url
        // in a `Url` but this will break SQLite on Windows, as attempting to
        // parse a SQLite URL like:
        //
        // sqlite:///C:/Users/user/repo/repo.db3
        //
        // Will actually remove the trailing ':' from the disk letter.

        let sqlite_url = format!(
            "sqlite://{}?blob_storage_url={}",
            db_path.to_str().unwrap().replace("\\", "/"),
            urlencoding::encode(&blob_storage_url.to_string())
        );

        Ok(Self {
            directory: directory.as_ref().to_owned(),
            sql_repository_index: SqlIndexBackend::new(sqlite_url)?,
        })
    }

    pub async fn close(&mut self) {
        self.sql_repository_index.close().await;
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
                    return Err(Error::index_already_exists(
                        self.directory.display().to_string(),
                    ));
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

        info!(
            "Creating SQLite database at {}",
            self.sql_repository_index.url()
        );

        self.sql_repository_index.create_index().await
    }

    async fn destroy_index(&self) -> Result<()> {
        self.sql_repository_index.destroy_index().await?;

        tokio::fs::remove_dir_all(&self.directory)
            .await
            .map_other_err(format!(
                "failed to destroy repository root at `{}`",
                &self.directory.display()
            ))
    }

    async fn index_exists(&self) -> Result<bool> {
        self.sql_repository_index.index_exists().await
    }

    async fn register_workspace(
        &self,
        workspace_registration: &WorkspaceRegistration,
    ) -> Result<()> {
        self.sql_repository_index
            .register_workspace(workspace_registration)
            .await
    }

    async fn find_branch(&self, branch_name: &str) -> Result<Option<Branch>> {
        self.sql_repository_index.find_branch(branch_name).await
    }

    async fn read_branches(&self) -> Result<Vec<Branch>> {
        self.sql_repository_index.read_branches().await
    }

    async fn insert_branch(&self, branch: &Branch) -> Result<()> {
        self.sql_repository_index.insert_branch(branch).await
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        self.sql_repository_index.update_branch(branch).await
    }

    async fn find_branches_in_lock_domain(&self, lock_domain_id: &str) -> Result<Vec<Branch>> {
        self.sql_repository_index
            .find_branches_in_lock_domain(lock_domain_id)
            .await
    }

    async fn read_commit(&self, commit_id: &str) -> Result<Commit> {
        self.sql_repository_index.read_commit(commit_id).await
    }

    async fn insert_commit(&self, commit: &Commit) -> Result<()> {
        self.sql_repository_index.insert_commit(commit).await
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<()> {
        self.sql_repository_index
            .commit_to_branch(commit, branch)
            .await
    }

    async fn commit_exists(&self, commit_id: &str) -> Result<bool> {
        self.sql_repository_index.commit_exists(commit_id).await
    }

    async fn read_tree(&self, id: &str) -> Result<Tree> {
        self.sql_repository_index.read_tree(id).await
    }

    async fn save_tree(&self, tree: &Tree) -> Result<String> {
        self.sql_repository_index.save_tree(tree).await
    }

    async fn insert_lock(&self, lock: &Lock) -> Result<()> {
        self.sql_repository_index.insert_lock(lock).await
    }

    async fn find_lock(
        &self,
        lock_domain_id: &str,
        canonical_relative_path: &str,
    ) -> Result<Option<Lock>> {
        self.sql_repository_index
            .find_lock(lock_domain_id, canonical_relative_path)
            .await
    }

    async fn find_locks_in_domain(&self, lock_domain_id: &str) -> Result<Vec<Lock>> {
        self.sql_repository_index
            .find_locks_in_domain(lock_domain_id)
            .await
    }

    async fn clear_lock(&self, lock_domain_id: &str, canonical_relative_path: &str) -> Result<()> {
        self.sql_repository_index
            .clear_lock(lock_domain_id, canonical_relative_path)
            .await
    }

    async fn count_locks_in_domain(&self, lock_domain_id: &str) -> Result<i32> {
        self.sql_repository_index
            .count_locks_in_domain(lock_domain_id)
            .await
    }

    async fn get_blob_storage_url(&self) -> Result<BlobStorageUrl> {
        self.sql_repository_index.get_blob_storage_url().await
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
