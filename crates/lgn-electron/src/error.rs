use std::{path::PathBuf, process::ExitStatus};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Binary not found: {0}")]
    BinaryNotFound(#[from] which::Error),

    #[error("Couldn't run electron command: {0}")]
    ElectronCommand(#[from] std::io::Error),

    #[error("Electron command returned the non-0 exit status {0}")]
    ElectronCommandFailed(ExitStatus),

    #[error("Provided path \"{0}\" couldn't be canonicalized")]
    InvalidPath(PathBuf),

    #[error("Provided path \"{0}\" is not a directory")]
    PathIsNotADir(PathBuf),

    #[error("Provided path \"{0}\" is not a file")]
    PathIsNotAFile(PathBuf),

    #[error("Command {0} is not implemented yet")]
    UnimplementedCommand(String),

    #[error("Electron runtime configuration is invalid")]
    InvalidElectronRuntimeConfiguration,
}

pub type Result<T> = std::result::Result<T, Error>;
