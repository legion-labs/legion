use std::{collections::BTreeSet, path::PathBuf};

use lgn_content_store::{indexing::IndexKey, Identifier};
use thiserror::Error;

use crate::{Branch, CanonicalPath, CommitId, Lock, RepositoryName};

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid repository name `{repository_name}`: {reason}")]
    InvalidRepositoryName {
        repository_name: String,
        reason: String,
    },
    #[error("the specified repository `{repository_name}` does not exist")]
    RepositoryDoesNotExist { repository_name: RepositoryName },
    #[error("the specified repository `{repository_name}` already exists")]
    RepositoryAlreadyExists { repository_name: RepositoryName },
    #[error("commit `{commit_id}` was not found")]
    CommitNotFound { commit_id: CommitId },
    #[error("the folder `{path}` is not a workspace")]
    NotAWorkspace { path: PathBuf },
    #[error("the path `{path}` did not match any files")]
    UnmatchedPath { path: PathBuf },
    #[error("the path `{path}` is a symbolic link which is not supported")]
    SymbolicLinkNotSupported { path: PathBuf },
    #[error("branch `{branch_name}` was not found")]
    BranchNotFound { branch_name: String },
    #[error("lock `{lock_domain_id}/{canonical_path}` was not found")]
    LockNotFound {
        lock_domain_id: String,
        canonical_path: CanonicalPath,
    },
    #[error("already on branch `{branch_name}`")]
    AlreadyOnBranch { branch_name: String },
    #[error("the workspace is dirty - please commit or stash changes")]
    WorkspaceDirty,
    #[error("cannot commit on stale branch `{}` who is now at `{}`", .branch.name, .branch.head)]
    StaleBranch { branch: Branch },
    #[error("cannot sync with conflicting changes")]
    ConflictingChanges,
    #[error("lock `{lock}` already exists")]
    LockAlreadyExists { lock: Lock },
    #[error("empty commits are not allowed: have you forgotten to stage your changes?")]
    EmptyCommitNotAllowed,
    #[error("some files are marked for edition but no changes are staged for them: please stage your changes or revert the files")]
    UnchangedFilesMarkedForEdition { paths: BTreeSet<CanonicalPath> },
    #[error("`{canonical_path}` is a directory and cannot be edited")]
    CannotEditDirectory { canonical_path: CanonicalPath },
    #[error(
        "a file at `{canonical_path}` already exists with a different content and cannot be added"
    )]
    FileAlreadyExists { canonical_path: CanonicalPath },
    #[error("the file at `{canonical_path}` does not exist")]
    FileDoesNotExist { canonical_path: CanonicalPath },
    #[error("`{canonical_path}` is not a file")]
    PathIsNotAFile { canonical_path: CanonicalPath },
    #[error("`{canonical_path}` is not a directory")]
    PathIsNotADirectory { canonical_path: CanonicalPath },
    #[error("file content for `{canonical_path}` does not match what was expected: got `{}` but expected `{}`", .id, .expected_id)]
    FileContentMistmatch {
        canonical_path: CanonicalPath,
        expected_id: Identifier,
        id: Identifier,
    },
    #[error("invalid canonical path `{path}`: {reason}")]
    InvalidCanonicalPath { path: String, reason: String },
    #[error("invalid change type")]
    InvalidChangeType,
    #[error("invalid tree node")]
    InvalidTreeNode,
    #[error("online error: {0}")]
    Online(#[from] lgn_online::Error),
    #[error("configuration error: {0}")]
    Config(#[from] lgn_config::Error),
    #[error("{context}: {source}")]
    Other {
        #[source]
        source: anyhow::Error,
        context: String,
    },
    #[error("{0}")]
    Unspecified(String),
    #[error("content store indexing: {0}")]
    ContentStoreIndexing(#[from] lgn_content_store::indexing::Error),
    #[error("resource  `{id}` not found in content store")]
    ResourceNotFoundById { id: IndexKey },
    #[error("resource  `{path}` not found in content store")]
    ResourceNotFoundByPath { path: String },
    #[error("path `{path}` is not valid for storage in content store")]
    InvalidPath { path: String },
}

impl Error {
    pub fn repository_does_not_exist(repository_name: RepositoryName) -> Self {
        Self::RepositoryDoesNotExist { repository_name }
    }

    pub fn repository_already_exists(repository_name: RepositoryName) -> Self {
        Self::RepositoryAlreadyExists { repository_name }
    }

    pub fn commit_not_found(commit_id: CommitId) -> Self {
        Self::CommitNotFound { commit_id }
    }

    pub fn not_a_workspace(path: impl Into<PathBuf>) -> Self {
        Self::NotAWorkspace { path: path.into() }
    }

    pub fn unmatched_path(path: impl Into<PathBuf>) -> Self {
        Self::UnmatchedPath { path: path.into() }
    }

    pub fn symbolic_link_not_supported(path: impl Into<PathBuf>) -> Self {
        Self::SymbolicLinkNotSupported { path: path.into() }
    }

    pub fn branch_not_found(branch_name: String) -> Self {
        Self::BranchNotFound { branch_name }
    }

    pub fn lock_not_found(lock_domain_id: String, canonical_path: CanonicalPath) -> Self {
        Self::LockNotFound {
            lock_domain_id,
            canonical_path,
        }
    }

    pub fn already_on_branch(branch_name: String) -> Self {
        Self::AlreadyOnBranch { branch_name }
    }

    pub fn stale_branch(branch: Branch) -> Self {
        Self::StaleBranch { branch }
    }

    pub fn lock_already_exists(lock: Lock) -> Self {
        Self::LockAlreadyExists { lock }
    }

    pub fn unchanged_files_marked_for_edition(paths: BTreeSet<CanonicalPath>) -> Self {
        Self::UnchangedFilesMarkedForEdition { paths }
    }

    pub fn cannot_edit_directory(canonical_path: CanonicalPath) -> Self {
        Self::CannotEditDirectory { canonical_path }
    }

    pub fn file_already_exists(canonical_path: CanonicalPath) -> Self {
        Self::FileAlreadyExists { canonical_path }
    }

    pub fn file_does_not_exist(canonical_path: CanonicalPath) -> Self {
        Self::FileDoesNotExist { canonical_path }
    }

    pub fn path_is_not_a_file(canonical_path: CanonicalPath) -> Self {
        Self::PathIsNotAFile { canonical_path }
    }

    pub fn path_is_not_a_directory(canonical_path: CanonicalPath) -> Self {
        Self::PathIsNotADirectory { canonical_path }
    }

    pub fn file_content_mismatch(
        canonical_path: CanonicalPath,
        expected_id: Identifier,
        id: Identifier,
    ) -> Self {
        Self::FileContentMistmatch {
            canonical_path,
            expected_id,
            id,
        }
    }

    pub fn invalid_canonical_path(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidCanonicalPath {
            path: path.into(),
            reason: reason.into(),
        }
    }

    pub fn resource_not_found_by_id(id: impl Into<IndexKey>) -> Self {
        Self::ResourceNotFoundById { id: id.into() }
    }

    pub fn resource_not_found_by_path(path: impl Into<String>) -> Self {
        Self::ResourceNotFoundByPath { path: path.into() }
    }

    pub fn invalid_path(path: impl Into<String>) -> Self {
        Self::InvalidPath { path: path.into() }
    }

    /// Prepends the parent node name to the canonical path of some
    /// tree-specific errors.
    ///
    /// Used in conjunction with the `WithParent` trait, this is mostly
    /// useful when dealing with recursive tree methods.
    fn with_parent_name(mut self, parent_name: &str) -> Self {
        match &mut self {
            Self::CannotEditDirectory { canonical_path }
            | Self::FileAlreadyExists { canonical_path }
            | Self::FileDoesNotExist { canonical_path }
            | Self::PathIsNotAFile { canonical_path }
            | Self::PathIsNotADirectory { canonical_path }
            | Self::FileContentMistmatch { canonical_path, .. } => {
                *canonical_path = canonical_path.clone().prepend(parent_name);
            }
            _ => {}
        };

        self
    }

    /// Prepends the parent node path to the canonical path of some
    /// tree-specific errors.
    ///
    /// Used in conjunction with the `WithParent` trait, this is mostly
    /// useful when dealing with recursive tree methods.
    fn with_parent_path(mut self, parent_path: &CanonicalPath) -> Self {
        match &mut self {
            Self::CannotEditDirectory { canonical_path }
            | Self::FileAlreadyExists { canonical_path }
            | Self::FileDoesNotExist { canonical_path }
            | Self::PathIsNotAFile { canonical_path }
            | Self::PathIsNotADirectory { canonical_path }
            | Self::FileContentMistmatch { canonical_path, .. } => {
                *canonical_path = parent_path.join(canonical_path);
            }
            _ => {}
        };

        self
    }
}

pub trait MapOtherError<T> {
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

pub trait WithParentName<T> {
    fn with_parent_name(self, parent_name: &str) -> Result<T>;
    fn with_parent_path(self, parent_path: &CanonicalPath) -> Result<T>;
}

impl<T> WithParentName<T> for std::result::Result<T, Error> {
    fn with_parent_name(self, parent_name: &str) -> Self {
        self.map_err(|e| e.with_parent_name(parent_name))
    }

    fn with_parent_path(self, parent_path: &CanonicalPath) -> Self {
        self.map_err(|e| e.with_parent_path(parent_path))
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
