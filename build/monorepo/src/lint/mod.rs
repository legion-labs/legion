use lgn_tracing::span_fn;

use crate::{context::Context, Result};

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
}

#[span_fn]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let all = !args.rules_coverage && !args.dependencies;

    if all || args.rules_coverage {
        rules_coverage::run(ctx)?;
    }
    if all || args.dependencies {
        dependencies::run(ctx)?;
    }
    Ok(())
}
