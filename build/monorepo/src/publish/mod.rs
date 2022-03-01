use std::collections::HashMap;

//use lgn_tracing::span_fn;

use crate::{
    cargo::{BuildArgs, SelectedPackageArgs},
    context::Context,
    Result,
};

mod aws_lambda;
mod docker;
mod hash;
mod metadata;
mod package;
mod target;
mod zip;

use lgn_tracing::{set_max_level, LevelFilter};
use package::PublishPackage;
use target::PublishTarget;

#[derive(Debug, clap::Args, Default)]
pub struct Args {
    #[clap(flatten)]
    pub(crate) package_args: SelectedPackageArgs,
    #[clap(flatten)]
    pub(crate) build_args: BuildArgs,
    /// Do not distribute the executable stop at the building step
    #[clap(long)]
    build_only: bool,
    /// Tag the new version
    #[clap(long)]
    update_hash: bool,
    /// Force the distribution of the package
    #[clap(long)]
    force: bool,
    /// Do not run any action that changes any external state
    #[clap(long)]
    dry_run: bool,
}

//#[span_fn]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    if args.build_args.verbose > 0 {
        set_max_level(LevelFilter::Debug);
    }

    let selected_packages = args
        .package_args
        .to_selected_packages(ctx)?
        .flatten_bins(ctx)?;

    let mut hash_cache = HashMap::new();
    let dist_packages: Vec<_> = selected_packages
        .iter()
        .map(|name| {
            PublishPackage::new(
                ctx,
                ctx.package_graph()?
                    .workspace()
                    .member_by_name(name)
                    .unwrap(),
                &mut hash_cache,
            )
        })
        .collect::<Result<Vec<_>>>()?;

    for pkg in dist_packages {
        let args = Args {
            package_args: SelectedPackageArgs {
                package: vec![pkg.name().into()],
                ..SelectedPackageArgs::default()
            },
            build_args: args.build_args.clone(),
            build_only: args.build_only,
            force: args.force,
            dry_run: args.dry_run,
            update_hash: args.update_hash,
        };
        pkg.dist(ctx, &args)?;
    }
    Ok(())
}
