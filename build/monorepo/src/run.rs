// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use chrono::Utc;
use lgn_telemetry::trace_scope;

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
    /// Dump a chrome trace file if possible
    #[clap(long)]
    #[allow(clippy::option_option)]
    ctrace: Option<Option<String>>,
    #[clap(name = "ARGS", parse(from_os_str), last = true)]
    args: Vec<OsString>,
}

pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    trace_scope!();
    let mut pass_through_args = vec![];
    pass_through_args.extend(args.args.clone());

    let mut direct_args = vec![];
    args.build_args.add_args(&mut direct_args);

    let mut packages = args.package_args.to_selected_packages(ctx)?;
    let bin = args.build_args.bin.first();
    let mut trace_name = bin.map_or_else(
        || {
            args.package_args
                .package
                .first()
                .map_or_else(|| "app".to_owned(), ToOwned::to_owned)
        },
        ToOwned::to_owned,
    );
    if let Some(bin) = bin {
        packages.select_package_from_bin(bin.as_str(), ctx)?;
    }
    trace_name = format!(
        "trace-{}-{}.json",
        trace_name,
        Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
    )
    .replace(":", "-");
    println!("{}", &trace_name);
    let env = if let Some(trace_file) = &args.ctrace {
        vec![(
            "LGN_TRACE_FILE",
            Some(
                trace_file
                    .as_ref()
                    .map_or_else(|| trace_name.as_str(), String::as_str),
            ),
        )]
    } else {
        vec![]
    };
    let cmd = CargoCommand::Run {
        direct_args: &direct_args,
        args: &pass_through_args,
        env: env.as_slice(),
    };
    cmd.run_on_packages(ctx, &packages)
}
