use std::ffi::OsString;

use lgn_tracing::span_fn;

use crate::cargo::{BuildArgs, CargoCommand, SelectedPackageArgs};
use crate::context::Context;
use crate::Result;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(flatten)]
    pub(crate) package_args: SelectedPackageArgs,
    #[clap(flatten)]
    pub(crate) build_args: BuildArgs,
    /// Automatically apply lint suggestions. This flag implies `--no-deps`
    #[clap(long)]
    fix: bool,
    #[clap(name = "ARGS", parse(from_os_str), last = true)]
    args: Vec<OsString>,
}

#[span_fn]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let mut pass_through_args: Vec<OsString> = vec![];
    for lint in &ctx.config().clippy.deny {
        pass_through_args.push("-D".into());
        pass_through_args.push(lint.into());
    }
    for lint in &ctx.config().clippy.warn {
        pass_through_args.push("-W".into());
        pass_through_args.push(lint.into());
    }
    for lint in &ctx.config().clippy.allow {
        pass_through_args.push("-A".into());
        pass_through_args.push(lint.into());
    }
    pass_through_args.extend(args.args.clone());

    let mut direct_args = vec![];
    args.build_args.add_args(&mut direct_args);

    if args.fix {
        direct_args.push("--fix".into());
    }

    let cmd = CargoCommand::Clippy {
        direct_args: &direct_args,
        args: &pass_through_args,
    };
    let packages = args.package_args.to_selected_packages(ctx)?;
    cmd.run_on_packages(ctx, &packages)
}
