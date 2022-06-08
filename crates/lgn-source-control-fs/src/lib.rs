//! Implements a FUSE filesystem for the source control repository.
//!

use std::path::Path;

use anyhow::Result;

use lgn_source_control::Index;

#[cfg(not(target_os = "windows"))]
mod filesystem;
#[cfg(not(target_os = "windows"))]
mod inode_index;

/// Implements all the running logic, so that we can easily conditionally
/// compile it for UNIX systems only.
///
/// # Errors
///
/// This function will return an error if the filesystem cannot be mounted.
///
#[cfg_attr(
    windows,
    allow(
        unused_variables,
        unreachable_code,
        clippy::unused_async,
        clippy::unimplemented
    )
)]
pub async fn run(
    index: Box<dyn Index>,
    branch: String,
    mountpoint: impl AsRef<Path>,
) -> Result<()> {
    #[cfg(target_os = "windows")]
    unimplemented!("Windows does not support fuse");

    #[cfg(not(target_os = "windows"))]
    {
        use anyhow::Context;
        use filesystem::SourceControlFilesystem;
        use fuser::MountOption;
        use tokio::sync::Semaphore;

        let fs = SourceControlFilesystem::new(index, branch).await?;
        let options = vec![MountOption::RO, MountOption::FSName("hello".to_string())];

        let session = fuser::Session::new(fs, mountpoint.as_ref(), &options)
            .context("failed to create fuse session")?;

        let session = session
            .spawn()
            .context("failed to run fuse session in the background")?;

        let semaphore = Semaphore::new(0);
        let _permit = semaphore.acquire().await?;

        session.join();

        Ok(())
    }
}
