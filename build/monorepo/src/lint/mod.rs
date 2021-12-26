use crate::{context::Context, Result};

mod rules_coverage;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Determinator rules coverage
    #[clap(long)]
    rules_coverage: bool,
}

pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let all = !args.rules_coverage;

    if all || args.rules_coverage {
        rules_coverage::run(ctx)?;
    }
    Ok(())
}
