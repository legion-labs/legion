use lgn_tracing::span_fn;

use crate::{context::Context, Result};

use super::utils::NpmWorkspace;

#[span_fn]
pub fn run(ctx: &Context) -> Result<()> {
    let npm_workspace = NpmWorkspace::empty(ctx)?;

    npm_workspace.run_install()?;

    Ok(())
}
