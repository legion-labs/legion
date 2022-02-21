// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lgn_tracing::span_fn;

use crate::{context::Context, Error, Result};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Run in 'check' mode. Exits with 0 if all tools installed. Exits with 1 and if not, printing failed
    #[clap(long)]
    check: bool,
    #[clap(long)]
    root: Option<String>,
}

#[span_fn]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let success = if args.check {
        ctx.installer().check_all()
    } else {
        ctx.installer().install_all(args.root.as_deref())
    };
    if success {
        Ok(())
    } else {
        Err(Error::new("Failed to install tools"))
    }
}
