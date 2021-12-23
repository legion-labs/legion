use std::collections::BTreeSet;
use std::ffi::OsString;

use crate::cargo::{BuildArgs, CargoCommand, SelectedPackageArgs, SelectedPackages};
use crate::context::Context;
use crate::Result;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(flatten)]
    pub(crate) package_args: SelectedPackageArgs,
    #[clap(flatten)]
    pub(crate) build_args: BuildArgs,
    #[clap(name = "ARGS", parse(from_os_str), last = true)]
    args: Vec<OsString>,
}

pub fn run(args: &Args, ctx: Context) -> Result<()> {
    let pass_through_args = vec!["-D".into(), "warnings".into()];
    // TODO
    //let mut pass_through_args = vec!["-D".into(), "warnings".into()];
    //for lint in ctx.config().allowed_clippy_lints() {
    //    pass_through_args.push("-A".into());
    //    pass_through_args.push(lint.into());
    //}
    //for lint in ctx.config().warn_clippy_lints() {
    //    pass_through_args.push("-W".into());
    //    pass_through_args.push(lint.into());
    //}
    //pass_through_args.extend(args.args);

    let mut direct_args = vec![];
    args.build_args.add_args(&mut direct_args);

    let cmd = CargoCommand::Clippy {
        cargo_config: ctx.config().cargo_config(),
        direct_args: &direct_args,
        args: &pass_through_args,
    };
    // TODO
    //let packages = args.package_args.to_selected_packages(&ctx)?;
    let packages = SelectedPackages {
        excludes: BTreeSet::new(),
    };
    cmd.run_on_packages(&packages)
}
