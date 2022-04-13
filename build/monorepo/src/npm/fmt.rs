use lgn_tracing::span_fn;

use crate::{context::Context, Result};

use super::utils::NpmWorkspace;

#[derive(Debug, Default, clap::Args)]
pub struct Args {
    /// Name of a npm package (as per its package.json file)
    /// If provided only this npm package is formatted
    #[clap(long, short)]
    pub(crate) package: Option<String>,

    /// Doesn't format files but instead checks their format is valid
    #[clap(long, short)]
    pub(crate) check: bool,
}

#[span_fn]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let mut npm_workspace = NpmWorkspace::new(ctx)?;

    npm_workspace.load_all();

    npm_workspace.format(&args.package, args.check)?;

    Ok(())
}
