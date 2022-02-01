use lgn_tracing::span_fn;

use crate::{context::Context, Result};

mod crate_attributes;
mod crate_files;
mod dependencies;
mod rules_coverage;

#[derive(Debug, clap::Args, Default)]
pub struct Args {
    /// Determinator rules coverage
    #[clap(long)]
    pub(crate) rules_coverage: bool,
    /// Run dependencies lints
    #[clap(long)]
    pub(crate) dependencies: bool,
    /// Run crate naming lints
    #[clap(long)]
    pub(crate) crate_attributes: bool,
    /// Run crate file lints
    #[clap(long)]
    pub(crate) crate_files: bool,
}

#[span_fn]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let all =
        !args.rules_coverage && !args.dependencies && !args.crate_attributes && !args.crate_files;

    if all || args.rules_coverage {
        rules_coverage::run(ctx)?;
    }
    if all || args.dependencies {
        dependencies::run(ctx)?;
    }
    // TODO: include in all checks when folders are renamed
    if
    /*all ||*/
    args.crate_attributes {
        crate_attributes::run(ctx)?;
    }
    if all || args.crate_files {
        crate_files::run(ctx)?;
    }

    Ok(())
}
