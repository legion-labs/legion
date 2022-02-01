// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lgn_tracing::span_fn;

use crate::{
    cargo::{CargoCommand, SelectedPackageArgs},
    context::Context,
    Result,
};
use std::ffi::OsString;

#[derive(Debug, Default, clap::Args)]
pub struct Args {
    #[clap(flatten)]
    pub(crate) package_args: SelectedPackageArgs,
    /// Do not run the benchmarks, but compile them
    #[clap(long)]
    pub(crate) no_run: bool,
    #[clap(name = "BENCHNAME", parse(from_os_str))]
    pub(crate) benchname: Option<OsString>,
    #[clap(name = "ARGS", parse(from_os_str), last = true)]
    pub(crate) args: Vec<OsString>,
}

#[span_fn]
pub fn run(mut args: Args, ctx: &Context) -> Result<()> {
    args.args.extend(args.benchname);

    let mut direct_args = Vec::new();
    if args.no_run {
        direct_args.push(OsString::from("--no-run"));
    };

    let cmd = CargoCommand::Bench {
        direct_args: direct_args.as_slice(),
        args: &args.args,
        env: &[],
    };

    let packages = args.package_args.to_selected_packages(ctx)?;
    cmd.run_on_packages(ctx, &packages)
}
