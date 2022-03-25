use lgn_tracing::span_fn;

use crate::{context::Context, Result};

use super::utils::NpmWorkspace;

#[derive(Debug, Default, clap::Args)]
pub struct Args {
    /// Name of a npm package (as per its package.json file)
    /// If provided only this npm package is built
    #[clap(long, short)]
    pub(crate) package: Option<String>,
    /// If this flag is present the dependencies will be installed
    #[clap(long, short)]
    pub(crate) install: bool,
    /// Forces the build
    #[clap(long, short)]
    pub(crate) force: bool,
}

#[span_fn]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let mut npm_workspace = NpmWorkspace::new(ctx)?;

    npm_workspace.load_all();

    if args.install {
        npm_workspace.install();
    }

    npm_workspace.build(&args.package, args.force)?;

    Ok(())
}
