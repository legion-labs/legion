use lgn_tracing::span_fn;

use crate::{context::Context, Result};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(long)]
    force: bool,
}

#[allow(clippy::unnecessary_wraps)]
#[span_fn]
pub fn run(_args: &Args, _ctx: &Context) -> Result<()> {
    Ok(())
}
