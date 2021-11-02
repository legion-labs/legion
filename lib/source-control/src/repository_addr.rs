use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RepositoryAddr {
    Local(PathBuf),
    Remote(String),
}

impl std::fmt::Display for RepositoryAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryAddr::Local(dir) => {
                write!(f, "{}", dir.display())
            }
            RepositoryAddr::Remote(uri) => {
                write!(f, "{}", uri)
            }
        }
    }
}
