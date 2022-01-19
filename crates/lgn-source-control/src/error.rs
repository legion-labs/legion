use std::path::PathBuf;

use thiserror::Error;

use crate::{Branch, Lock};

#[derive(Error, Debug)]
pub enum Error {
    #[error("the specified index does not exist")]
    IndexDoesNotExist { url: String },
    #[error("the specified index already exists")]
    IndexAlreadyExists { url: String },
    #[error("invalid index URL `{url}`: {source}")]
    InvalidIndexUrl {
        url: String,
        #[source]
        source: anyhow::Error,
    },
    #[error("the folder `{path}` is not a workspace")]
    NotAWorkspace { path: PathBuf },
    #[error("branch `{branch_name}` was not found")]
    BranchNotFound { branch_name: String },
    #[error("cannot commit on stale branch `{}` who is now at `{}`", .branch.name, .branch.head)]
    StaleBranch { branch: Branch },
    #[error("lock `{}` already exists in domain `{}`", .lock.relative_path, .lock.lock_domain_id)]
    LockAlreadyExists { lock: Lock },
    #[error("the directory `{path}` already exists and is not empty")]
    DirectoryAlreadyExists { path: PathBuf },
    #[error("{context}: {source}")]
    Other {
        #[source]
        source: anyhow::Error,
        context: String,
    },
}

impl Error {
    pub fn index_does_not_exist(url: impl Into<String>) -> Self {
        Self::IndexDoesNotExist { url: url.into() }
    }

    pub fn index_already_exists(url: impl Into<String>) -> Self {
        Self::IndexAlreadyExists { url: url.into() }
    }

    pub fn invalid_index_url(url: impl Into<String>, source: impl Into<anyhow::Error>) -> Self {
        Self::InvalidIndexUrl {
            url: url.into(),
            source: source.into(),
        }
    }

    pub fn not_a_workspace(path: impl Into<PathBuf>) -> Self {
        Self::NotAWorkspace { path: path.into() }
    }

    pub fn branch_not_found(branch_name: String) -> Self {
        Self::BranchNotFound { branch_name }
    }

    pub fn stale_branch(branch: Branch) -> Self {
        Self::StaleBranch { branch }
    }

    pub fn lock_already_exists(lock: Lock) -> Self {
        Self::LockAlreadyExists { lock }
    }

    pub fn directory_already_exists(path: PathBuf) -> Self {
        Self::DirectoryAlreadyExists { path }
    }
}

pub(crate) trait MapOtherError<T> {
    fn map_other_err(self, context: impl Into<String>) -> Result<T>;
}

impl<T, E> MapOtherError<T> for std::result::Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn map_other_err(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| Error::Other {
            context: context.into(),
            source: e.into(),
        })
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
