use async_trait::async_trait;

use crate::{
    utils::{parse_url_or_path, UrlOrPath},
    BlobStorageUrl, Branch, CanonicalPath, Commit, CommitId, Error, GrpcIndexBackend,
    LocalIndexBackend, Lock, MapOtherError, Result, SqlIndexBackend, Tree, WorkspaceRegistration,
};

/// The query options for the `list_branches` method.
#[derive(Default, Clone, Debug)]
pub struct ListBranchesQuery<'q> {
    pub lock_domain_id: Option<&'q str>,
}

/// The query options for the `list_commits` method.
#[derive(Default, Clone, Debug)]
pub struct ListCommitsQuery {
    pub commit_ids: Vec<CommitId>,
    pub depth: u32,
}

impl ListCommitsQuery {
    pub fn single(commit_id: CommitId) -> Self {
        Self {
            commit_ids: vec![commit_id],
            ..Self::default()
        }
    }
}

/// The query options for the `list_locks` method.
#[derive(Default, Clone, Debug)]
pub struct ListLocksQuery<'q> {
    pub lock_domain_ids: Vec<&'q str>,
}

#[async_trait]
pub trait IndexBackend: Send + Sync {
    fn url(&self) -> &str;
    async fn create_index(&self) -> Result<BlobStorageUrl>;
    async fn destroy_index(&self) -> Result<()>;
    async fn index_exists(&self) -> Result<bool>;

    async fn get_blob_storage_url(&self) -> Result<BlobStorageUrl>;

    async fn register_workspace(
        &self,
        workspace_registration: &WorkspaceRegistration,
    ) -> Result<()>;

    async fn insert_branch(&self, branch: &Branch) -> Result<()>;
    async fn update_branch(&self, branch: &Branch) -> Result<()>;
    async fn get_branch(&self, branch_name: &str) -> Result<Branch>;
    async fn list_branches(&self, query: &ListBranchesQuery<'_>) -> Result<Vec<Branch>>;

    async fn get_commit(&self, commit_id: CommitId) -> Result<Commit> {
        self.list_commits(&ListCommitsQuery {
            commit_ids: vec![commit_id],
            depth: 1,
        })
        .await?
        .pop()
        .ok_or_else(|| Error::commit_not_found(commit_id))
    }

    async fn list_commits(&self, query: &ListCommitsQuery) -> Result<Vec<Commit>>;
    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<CommitId>;

    async fn get_tree(&self, id: &str) -> Result<Tree>;
    async fn save_tree(&self, tree: &Tree) -> Result<String>;

    async fn lock(&self, lock: &Lock) -> Result<()>;
    async fn unlock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<()>;
    async fn get_lock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<Lock>;
    async fn list_locks(&self, query: &ListLocksQuery<'_>) -> Result<Vec<Lock>>;
    async fn count_locks(&self, query: &ListLocksQuery<'_>) -> Result<i32>;
}

pub fn new_index_backend(url: &str) -> Result<Box<dyn IndexBackend>> {
    Ok(
        match parse_url_or_path(url)
            .map_other_err(format!("failed to parse index url `{}`", &url))?
        {
            UrlOrPath::Path(path) => Box::new(LocalIndexBackend::new(path)?),
            UrlOrPath::Url(url) => match url.scheme() {
                "mysql" => Box::new(SqlIndexBackend::new(url.to_string())?),
                "http" | "https" => Box::new(GrpcIndexBackend::new(url)?),
                scheme => {
                    return Err(Error::invalid_index_url(
                        url.to_string(),
                        anyhow::anyhow!("unsupported repository URL scheme: {}", scheme),
                    ))
                }
            },
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_new_backend_from_str_file() {
        #[cfg(not(windows))]
        assert_eq!(
            new_index_backend("file:///home/user/repo").unwrap().url(),
            "/home/user/repo"
        );
        #[cfg(windows)]
        assert_eq!(
            new_index_backend(r"file:///C:/Users/user/repo")
                .unwrap()
                .url(),
            r"C:\Users\user\repo"
        );
        #[cfg(windows)]
        assert_eq!(
            new_index_backend(r"file:///C:\Users\user\repo")
                .unwrap()
                .url(),
            r"C:\Users\user\repo"
        );
    }

    #[test]
    fn test_index_new_backend_from_str_file_no_scheme() {
        #[cfg(not(windows))]
        assert_eq!(
            new_index_backend("/home/user/repo").unwrap().url(),
            "/home/user/repo"
        );
        #[cfg(windows)]
        assert_eq!(
            new_index_backend(r"C:/Users/user/repo").unwrap().url(),
            r"C:/Users/user/repo"
        );
        #[cfg(windows)]
        assert_eq!(
            new_index_backend(r"C:\Users\user\repo").unwrap().url(),
            r"C:\Users\user\repo"
        );
    }

    #[test]
    fn test_index_new_backend_from_str_file_no_scheme_relative() {
        assert!(new_index_backend("repo").is_err());
    }

    #[test]
    fn test_index_new_backend_from_str_mysql() {
        assert_eq!(
            new_index_backend("mysql://user:pass@localhost:3306/db?blob_storage_url=blobs")
                .unwrap()
                .url(),
            "mysql://user:pass@localhost:3306/db"
        );
    }

    #[test]
    #[should_panic]
    fn test_index_new_backend_from_str_mysql_missing_blob_storage_url() {
        assert_eq!(
            new_index_backend("mysql://user:pass@localhost:3306/db")
                .unwrap()
                .url(),
            "mysql://user:pass@localhost:3306/db"
        );
    }

    #[test]
    fn test_index_new_backend_from_str_grpc() {
        assert_eq!(
            new_index_backend("http://user:pass@localhost:3306/db")
                .unwrap()
                .url(),
            "http://user:pass@localhost:3306/db"
        );
        assert_eq!(
            new_index_backend("https://user:pass@localhost:3306/db")
                .unwrap()
                .url(),
            "https://user:pass@localhost:3306/db"
        );
    }

    #[test]
    #[should_panic]
    fn test_index_new_backend_from_str_unsupported() {
        new_index_backend("file:").unwrap();
    }
}
