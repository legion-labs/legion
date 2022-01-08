// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lgn_tracing::trace_function;

use crate::{
    cargo::{BuildArgs, CargoCommand, SelectedPackageArgs},
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

#[trace_function]
pub fn run(mut args: Args, ctx: &Context) -> Result<()> {
    let mut pass_through_args = vec![];
    pass_through_args.extend(args.args.clone());

    // Always run fix on all targets.
    args.build_args.all_targets = true;

    let mut direct_args = vec![];
    args.build_args.add_args(&mut direct_args);

    let cmd = CargoCommand::Fix {
        direct_args: &direct_args,
        args: &pass_through_args,
    };
    let packages = args.package_args.to_selected_packages(ctx)?;
    cmd.run_on_packages(ctx, &packages)
}
