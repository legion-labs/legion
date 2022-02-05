use lgn_tracing::span_fn;

use crate::{context::Context, Result};

use super::utils::NpmWorkspace;

#[derive(Debug, Default, clap::Args)]
pub struct Args {
    /// Name of a npm package (as per its package.json file)
    /// If provided only this npm package is built
    #[clap(long, short)]
    pub(crate) package: Option<String>,
    /// If this flag is present the dependencies will not be installed
    #[clap(long, short)]
    pub(crate) no_install: bool,
}

#[span_fn]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let npm_workspace = NpmWorkspace::new(ctx)?;

    if !args.no_install {
        npm_workspace.run_install()?;
    }

    npm_workspace.run_build(&args.package)?;

    Ok(())
}
