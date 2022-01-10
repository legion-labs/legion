use std::ffi::OsString;

use lgn_tracing::span_fn;

use crate::cargo::{BuildArgs, CargoCommand, SelectedPackageArgs};
use crate::context::Context;
use crate::{Error, Result};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(flatten)]
    pub(crate) package_args: SelectedPackageArgs,
    #[clap(flatten)]
    pub(crate) build_args: BuildArgs,
    /// Open the docs in a browser after building them
    #[clap(long)]
    pub(crate) open: bool,
    /// Do not build documentation for dependencies.
    #[clap(long)]
    pub(crate) no_deps: bool,
    /// Include non-public items in the documentation.\
    #[clap(long)]
    pub(crate) document_private_items: bool,
    #[clap(name = "ARGS", parse(from_os_str), last = true)]
    args: Vec<OsString>,
}

#[span_fn]
pub fn run(mut args: Args, ctx: &Context) -> Result<()> {
    // Force no deps
    args.no_deps = true;

    let mut rustodc_flags: Vec<String> = vec![];
    for lint in &ctx.config().rustdoc.deny {
        rustodc_flags.push("-D".into());
        rustodc_flags.push(lint.into());
    }
    for lint in &ctx.config().rustdoc.warn {
        rustodc_flags.push("-W".into());
        rustodc_flags.push(lint.into());
    }
    for lint in &ctx.config().rustdoc.allow {
        rustodc_flags.push("-A".into());
        rustodc_flags.push(lint.into());
    }
    let rustodc_flags = rustodc_flags.join(" ");

    let mut direct_args = vec![];
    args.build_args.add_args(&mut direct_args);
    if args.open {
        direct_args.push(OsString::from("--open"));
    }
    if args.no_deps {
        direct_args.push(OsString::from("--no-deps"));
    }
    if args.document_private_items {
        direct_args.push(OsString::from("--document-private-items"));
    }
    let cmd = CargoCommand::Doc {
        direct_args: &direct_args,
        args: &args.args,
        env: &[("RUSTDOCFLAGS", Some(&rustodc_flags))],
    };
    let packages = args.package_args.to_selected_packages(ctx)?;
    cmd.run_on_packages(ctx, &packages)?;

    if packages.includes_package(&ctx.config().rustdoc.entry_point) {
        std::fs::write(
            ctx.workspace_root().join("target/doc/index.html"),
            format!(
                "<meta http-equiv=\"refresh\" content=\"0; URL={}/index.html\"/>",
                ctx.config().rustdoc.entry_point.replace("-", "_")
            ),
        )
        .map_err(|err| Error::new("Could not write entry point").with_source(err))?;
    }
    Ok(())
}
