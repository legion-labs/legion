// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::{
    cargo::{BuildArgs, CargoCommand, SelectedPackageArgs},
    context::Context,
    Result,
};
use lgn_tracing::{info, span_fn};
use std::ffi::OsString;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(flatten)]
    package_args: SelectedPackageArgs,
    #[clap(flatten)]
    build_args: BuildArgs,
    /// Copy final artifacts to this directory (unstable)
    #[clap(long, parse(from_os_str))]
    out_dir: Option<OsString>,
    /// Output the build plan in JSON (unstable)
    #[clap(long)]
    build_plan: bool,
}

pub fn convert_args(args: &Args) -> Vec<OsString> {
    let mut direct_args = Vec::new();
    args.build_args.add_args(&mut direct_args);
    if let Some(out_dir) = &args.out_dir {
        direct_args.push(OsString::from("--out-dir"));
        direct_args.push(OsString::from(out_dir));
    };
    if args.build_plan {
        direct_args.push(OsString::from("--build-plan"));
    };

    direct_args
}

#[span_fn]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    info!("Build plan: {}", args.build_plan);

    let direct_args = convert_args(args);

    let cmd = CargoCommand::Build {
        direct_args: direct_args.as_slice(),
        args: &[],
        env: &[],
        skip_sccache: false,
    };

    let packages = args.package_args.to_selected_packages(ctx)?;
    cmd.run_on_packages(ctx, &packages)
}
