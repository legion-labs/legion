use lgn_tracing::trace_function;

use crate::{context::Context, Result};

mod dependencies;
mod rules_coverage;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Determinator rules coverage
    #[clap(long)]
    rules_coverage: bool,
    /// Run dependencies lints
    #[clap(long)]
    dependencies: bool,
}

#[trace_function]
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
