// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lgn_tracing::span_fn;

use crate::{cargo::Cargo, context::Context, Result};
use std::ffi::OsString;

#[derive(Debug, clap::Args, Default)]
pub struct Args {
    #[clap(long)]
    /// Run in 'check' mode. Exits with 0 if input is
    /// formatted correctly. Exits with 1 and prints a diff if
    /// formatting is required.
    pub(crate) check: bool,

    #[clap(long)]
    /// Run check on all packages in the workspace
    pub(crate) workspace: bool,

    #[clap(name = "ARGS", parse(from_os_str), last = true)]
    /// Pass through args to rustfmt
    pub(crate) args: Vec<OsString>,
}

#[span_fn]
pub fn run(args: Args, ctx: &Context) -> Result<()> {
    let mut pass_through_args = vec![];

    if args.check {
        pass_through_args.push("--check".into());
    }

    pass_through_args.extend(args.args);

    let mut cmd = Cargo::new(ctx, "fmt", true);

    if args.workspace {
        // cargo fmt doesn't have a --workspace flag, instead it uses the
        // old --all flag
        cmd.all();
    }

    cmd.pass_through(&pass_through_args).run()
}
