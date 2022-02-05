use lgn_tracing::span_fn;

use crate::{context::Context, Result};

use super::utils::NpmWorkspace;

#[derive(Debug, Default, clap::Args)]
pub struct Args {
    /// Name of a npm package (as per its package.json file)
    /// If provided only this npm package is checked
    #[clap(long, short)]
    pub(crate) package: Option<String>,
}

#[span_fn]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let npm_workspace = NpmWorkspace::new(ctx)?;

    npm_workspace.run_check(&args.package)?;

    Ok(())
}
