use lgn_tracing::span_fn;

use crate::{context::Context, Result};

use super::utils::NpmWorkspace;

#[derive(Debug, Default, clap::Args)]
pub struct Args {
    /// Name of a npm package (as per its package.json file)
    /// If provided only this npm package is checked
    #[clap(long, short)]
    pub(crate) package: Option<String>,
    /// Skips the build step, resulting in a faster but
    /// less reliable check
    #[clap(long, short)]
    pub(crate) skip_build: bool,
    /// First install npm packages
    #[clap(long)]
    pub(crate) npm_install: bool,
}

#[span_fn]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let mut npm_workspace = NpmWorkspace::new(ctx)?;

    npm_workspace.load_all();

    if !npm_workspace.is_empty() {
        if args.npm_install {
            npm_workspace.install();
        }

        if !args.skip_build {
            npm_workspace.build(&args.package, true)?;
        }

        npm_workspace.check(&args.package)?;
    }
    Ok(())
}
