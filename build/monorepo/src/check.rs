// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lgn_tracing::trace_function;

use crate::{
    cargo::{BuildArgs, CargoCommand, SelectedPackageArgs},
    context::Context,
    Result,
};
#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(flatten)]
    pub(crate) package_args: SelectedPackageArgs,
    #[clap(flatten)]
    pub(crate) build_args: BuildArgs,
}

#[trace_function]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let mut direct_args = vec![];
    args.build_args.add_args(&mut direct_args);

    let cmd = CargoCommand::Check {
        direct_args: &direct_args,
    };
    let packages = args.package_args.to_selected_packages(ctx)?;
    cmd.run_on_packages(ctx, &packages)
}
