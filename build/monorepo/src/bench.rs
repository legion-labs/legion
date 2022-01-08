// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lgn_tracing::trace_function;

use crate::{
    cargo::{CargoCommand, SelectedPackageArgs},
    context::Context,
    Result,
};
use std::ffi::OsString;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(flatten)]
    package_args: SelectedPackageArgs,
    /// Do not run the benchmarks, but compile them
    #[clap(long)]
    no_run: bool,
    #[clap(name = "BENCHNAME", parse(from_os_str))]
    benchname: Option<OsString>,
    #[clap(name = "ARGS", parse(from_os_str), last = true)]
    args: Vec<OsString>,
}

#[trace_function]
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
