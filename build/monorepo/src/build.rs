// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::{
    cargo::{BuildArgs, CargoCommand, SelectedPackageArgs},
    context::Context,
    npm::utils::NpmWorkspace,
    Result,
};

use lgn_tracing::{info, span_fn};
use std::ffi::OsString;

#[derive(Debug, clap::Args, Default)]
pub struct Args {
    #[clap(flatten)]
    pub(crate) package_args: SelectedPackageArgs,
    #[clap(flatten)]
    pub(crate) build_args: BuildArgs,
    /// Copy final artifacts to this directory (unstable)
    #[clap(long, parse(from_os_str))]
    pub(crate) out_dir: Option<OsString>,
    /// Output the build plan in JSON (unstable)
    #[clap(long)]
    pub(crate) build_plan: bool,
    /// Skip npm packages build,
    /// this will also skip the npm install step
    /// even if the --npm-install flag is present
    #[clap(long)]
    pub(crate) skip_npm_build: bool,
    /// Forces the associated npm package build
    /// Doesn't work if `--skip-npm-build` is used
    #[clap(long, short)]
    pub(crate) force_npm_build: bool,
    /// First install npm packages
    #[clap(long)]
    pub(crate) npm_install: bool,
}

#[span_fn]
pub fn run(mut args: Args, ctx: &Context) -> Result<()> {
    info!("Build plan: {}", args.build_plan);

    let mut direct_args = Vec::new();

    args.build_args.add_args(&mut direct_args);
    if let Some(out_dir) = &args.out_dir {
        direct_args.push(OsString::from("--out-dir"));
        direct_args.push(OsString::from(out_dir));
    };
    if args.build_plan {
        direct_args.push(OsString::from("--build-plan"));
    };

    let mut env = vec![];
    args.build_args.add_env(&mut env);
    let cmd = CargoCommand::Build {
        direct_args: direct_args.as_slice(),
        args: &[],
        env: &env,
        skip_sccache: false,
    };

    // exclude the package itself
    args.package_args
        .exclude
        .push(env!("CARGO_PKG_NAME").into());

    let mut packages = args.package_args.to_selected_packages(ctx)?;
    if let Some(bin) = args.build_args.bin.first() {
        packages.select_package_from_bin(bin.as_str(), ctx)?;
    } else if let Some(example) = args.build_args.example.first() {
        packages.select_package_from_example(example.as_str(), ctx)?;
    }

    // Npm packages related code
    if !args.skip_npm_build {
        let mut npm_workspace = NpmWorkspace::new(ctx)?;

        npm_workspace.load_selected_packages(&packages)?;

        if !npm_workspace.is_empty() {
            if args.npm_install {
                npm_workspace.install();
            }

            npm_workspace.build(&None, args.force_npm_build)?;
        }
    }

    cmd.run_on_packages(ctx, &packages)
}
