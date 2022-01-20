use async_trait::async_trait;

use crate::{
    utils::{parse_url_or_path, UrlOrPath},
    BlobStorageUrl, Branch, Commit, Error, GrpcIndexBackend, LocalIndexBackend, Lock,
    MapOtherError, Result, SqlIndexBackend, Tree, WorkspaceRegistration,
};

#[async_trait]
pub trait IndexBackend: Send + Sync {
    fn url(&self) -> &str;
    async fn create_index(&self) -> Result<BlobStorageUrl>;
    async fn destroy_index(&self) -> Result<()>;
    async fn index_exists(&self) -> Result<bool>;
    async fn register_workspace(
        &self,
        workspace_registration: &WorkspaceRegistration,
    ) -> Result<()>;
    async fn read_branch(&self, branch_name: &str) -> Result<Branch> {
        self.find_branch(branch_name)
            .await?
            .ok_or_else(|| Error::BranchNotFound {
                branch_name: branch_name.to_string(),
            })
    }
    async fn insert_branch(&self, branch: &Branch) -> Result<()>;
    async fn update_branch(&self, branch: &Branch) -> Result<()>;
    async fn find_branch(&self, branch_name: &str) -> Result<Option<Branch>>;
    async fn find_branches_in_lock_domain(&self, lock_domain_id: &str) -> Result<Vec<Branch>>;
    async fn read_branches(&self) -> Result<Vec<Branch>>;
    async fn read_commit(&self, commit_id: &str) -> Result<Commit>;
    async fn insert_commit(&self, commit: &Commit) -> Result<()>;
    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<()>;
    async fn commit_exists(&self, commit_id: &str) -> Result<bool>;
    async fn read_tree(&self, id: &str) -> Result<Tree>;
    async fn save_tree(&self, tree: &Tree) -> Result<String>;
    async fn insert_lock(&self, lock: &Lock) -> Result<()>;
    async fn find_lock(&self, lock_domain_id: &str, relative_path: &str) -> Result<Option<Lock>>;
    async fn find_locks_in_domain(&self, lock_domain_id: &str) -> Result<Vec<Lock>>;
    async fn clear_lock(&self, lock_domain_id: &str, relative_path: &str) -> Result<()>;
    async fn count_locks_in_domain(&self, lock_domain_id: &str) -> Result<i32>;
    async fn get_blob_storage_url(&self) -> Result<BlobStorageUrl>;
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
