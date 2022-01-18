use std::path::Path;

use anyhow::{Context, Result};

use lgn_source_control::IndexBackend;

#[cfg(not(target_os = "windows"))]
mod filesystem;

#[cfg(not(target_os = "windows"))]
use filesystem::SourceControlFilesystem;
#[cfg(not(target_os = "windows"))]
use fuser::MountOption;
use tokio::sync::Semaphore;

/// Implements all the running logic, so that we can easily conditionally
/// compile it for UNIX systems only.
///
/// # Errors
///
/// This function will return an error if the filesystem cannot be mounted.
pub async fn run(
    index_backend: Box<dyn IndexBackend>,
    branch: String,
    mountpoint: impl AsRef<Path>,
) -> Result<()> {
    #[cfg(target_os = "windows")]
    unimplemented!("Windows does not support fuse");

    #[cfg(not(target_os = "windows"))]
    {
        let fs = SourceControlFilesystem::new(index_backend, branch);
        let options = vec![MountOption::RO, MountOption::FSName("hello".to_string())];

        let session = fuser::Session::new(fs, mountpoint.as_ref(), &options)
            .context("failed to create fuse session")?;

        let _session = session
            .spawn()
            .context("failed to run fuse session in the background")?;

        let semaphore = Semaphore::new(0);
        let _permit = semaphore.acquire().await?;
    }

    Ok(())
}
