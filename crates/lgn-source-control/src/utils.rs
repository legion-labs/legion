use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub(crate) fn make_path_absolute(path: impl AsRef<Path>) -> Result<PathBuf> {
    //fs::canonicalize is a trap - it generates crazy unusable "extended length" paths
    let path = path.as_ref();

    Ok(if path.is_absolute() {
        PathBuf::from(path_clean::clean(
            path.to_str().context("failed to convert path to string")?,
        ))
    } else {
        PathBuf::from(path_clean::clean(
            std::env::current_dir()
                .context("failed to get current directory")?
                .join(path)
                .to_str()
                .context("failed to convert path to string")?,
        ))
    })
}
