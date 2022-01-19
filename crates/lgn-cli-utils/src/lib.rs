//! Legion cli crate, contains CLI utilities.

// crate-specific lint exceptions:
#![allow(clippy::implicit_hasher, clippy::missing_errors_doc)]

pub(crate) mod async_reverse_single_lock;
pub mod termination_handler;

/// Wait for a signal to terminate the process.
pub async fn wait_for_termination() -> anyhow::Result<()> {
    let handler = termination_handler::AsyncTerminationHandler::new()?;

    handler.wait().await;

    Ok(())
}
