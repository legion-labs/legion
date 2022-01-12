use std::path::PathBuf;

use thiserror::Error;

use crate::{blob_storage::BlobStorageUrl, Branch, Lock};

#[derive(Error, Debug)]
pub enum Error {
    #[error("branch `{branch_name}` was not found")]
    BranchNotFound { branch_name: String },
    #[error("no blob storage URL was specified")]
    NoBlobStorageUrl,
    #[error("a blob storage URL was specified")]
    UnexpectedBlobStorageUrl { blob_storage_url: BlobStorageUrl },
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
    pub fn branch_not_found(branch_name: String) -> Self {
        Self::BranchNotFound { branch_name }
    }

    pub fn no_blob_storage_url() -> Self {
        Self::NoBlobStorageUrl
    }

    pub fn unexpected_blob_storage_url(blob_storage_url: BlobStorageUrl) -> Self {
        Self::UnexpectedBlobStorageUrl { blob_storage_url }
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
