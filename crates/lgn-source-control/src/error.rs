use std::{collections::BTreeSet, path::PathBuf};

use thiserror::Error;

use crate::{Branch, CanonicalPath, FileInfo, Lock};

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
    #[error("the specified commit `{commit_id}` does not exist")]
    CommitDoesNotExist { commit_id: String },
    #[error("the folder `{path}` is not a workspace")]
    NotAWorkspace { path: PathBuf },
    #[error("the path `{path}` did not match any files")]
    UnmatchedPath { path: PathBuf },
    #[error("the path `{path}` is a symbolic link which is not supported")]
    SymbolicLinkNotSupported { path: PathBuf },
    #[error("branch `{branch_name}` was not found")]
    BranchNotFound { branch_name: String },
    #[error("already on branch `{branch_name}`")]
    AlreadyOnBranch { branch_name: String },
    #[error("the workspace is dirty - please commit or stash changes")]
    WorkspaceDirty,
    #[error("cannot commit on stale branch `{}` who is now at `{}`", .branch.name, .branch.head)]
    StaleBranch { branch: Branch },
    #[error("lock `{}` already exists in domain `{}`", .lock.relative_path, .lock.lock_domain_id)]
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
    #[error("file content for `{canonical_path}` does not match what was expected: got `{}` but expected `{}`", .info.hash, .expected_info.hash)]
    FileContentMistmatch {
        canonical_path: CanonicalPath,
        expected_info: FileInfo,
        info: FileInfo,
    },
    #[error("invalid canonical path `{path}`: {reason}")]
    InvalidCanonicalPath { path: String, reason: String },
    #[error("invalid change type")]
    InvalidChangeType,
    #[error("invalid tree node")]
    InvalidTreeNode,
    #[error("path `{path}` is not included")]
    PathNotIncluded { path: CanonicalPath },
    #[error("path `{path}` is excluded by `{exclusion_rule}`")]
    PathExcluded {
        path: CanonicalPath,
        exclusion_rule: CanonicalPath,
    },
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

    pub fn commit_does_not_exist(commit_id: impl Into<String>) -> Self {
        Self::CommitDoesNotExist {
            commit_id: commit_id.into(),
        }
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

    pub fn already_on_branch(branch_name: String) -> Self {
        Self::AlreadyOnBranch { branch_name }
    }

    pub fn stale_branch(branch: Branch) -> Self {
        Self::StaleBranch { branch }
    }

    pub fn lock_already_exists(lock: Lock) -> Self {
        Self::LockAlreadyExists { lock }
    }

    pub fn path_not_included(path: CanonicalPath) -> Self {
        Self::PathNotIncluded { path }
    }

    pub fn path_excluded(path: CanonicalPath, exclusion_rule: CanonicalPath) -> Self {
        Self::PathExcluded {
            path,
            exclusion_rule,
        }
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
        expected_info: FileInfo,
        info: FileInfo,
    ) -> Self {
        Self::FileContentMistmatch {
            canonical_path,
            expected_info,
            info,
        }
    }

    pub fn invalid_canonical_path(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidCanonicalPath {
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Prepends the parent node name to the canonical path of some
    /// tree-specific errors.
    ///
    /// Used in conjunction with the `WithParent` trait, this is mostly
    /// useful when dealing with recursive tree methods.
    fn with_parent_name(mut self, parent_name: &str) -> Self {
        if let Some(canonical_path) = match &mut self {
            Self::CannotEditDirectory { canonical_path }
            | Self::FileAlreadyExists { canonical_path }
            | Self::FileDoesNotExist { canonical_path }
            | Self::PathIsNotAFile { canonical_path }
            | Self::PathIsNotADirectory { canonical_path }
            | Self::FileContentMistmatch { canonical_path, .. } => Some(canonical_path),
            _ => None,
        } {
            *canonical_path = canonical_path.prepend(parent_name);
        }

        self
    }

    /// Prepends the parent node path to the canonical path of some
    /// tree-specific errors.
    ///
    /// Used in conjunction with the `WithParent` trait, this is mostly
    /// useful when dealing with recursive tree methods.
    fn with_parent_path(mut self, parent_path: &CanonicalPath) -> Self {
        if let Some(canonical_path) = match &mut self {
            Self::CannotEditDirectory { canonical_path }
            | Self::FileAlreadyExists { canonical_path }
            | Self::FileDoesNotExist { canonical_path }
            | Self::PathIsNotAFile { canonical_path }
            | Self::PathIsNotADirectory { canonical_path }
            | Self::FileContentMistmatch { canonical_path, .. } => Some(canonical_path),
            _ => None,
        } {
            *canonical_path = parent_path.join(canonical_path);
        }

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
