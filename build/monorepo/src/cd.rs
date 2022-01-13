use lgn_tracing::span_fn;

use crate::{
    cargo::{BuildArgs, SelectedPackageArgs},
    context::Context,
    Result,
};
use std::ffi::OsString;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(flatten)]
    pub(crate) package_args: SelectedPackageArgs,
    #[clap(flatten)]
    pub(crate) build_args: BuildArgs,
    #[clap(name = "ARGS", parse(from_os_str), last = true)]
    args: Vec<OsString>,
}

#[allow(clippy::unnecessary_wraps)]
#[span_fn]
pub fn run(_args: &Args, _ctx: &Context) -> Result<()> {
    Ok(())
}
