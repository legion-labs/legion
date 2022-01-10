// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::git::GitCli;
use crate::{context::Context, Error, Result};
use determinator::{Determinator, DeterminatorSet};
use guppy::graph::{cargo::CargoResolverVersion, DependencyDirection};
use lgn_tracing::dispatch::{flush_thread_buffer, init_thread_stream};
use lgn_tracing::{span_fn, span_scope, trace};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// List packages changed since this commit
    pub(crate) base: String,
}

#[span_fn]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let git_cli = ctx.git_cli().map_err(|err| {
        err.with_explanation("changed-since` must be run within a project cloned from a git repo.")
    })?;
    let determinator_set = changed_since_impl(git_cli, ctx, &args.base)?;
    {
        span_scope!("print_packages");
        for package in determinator_set
            .affected_set
            .packages(DependencyDirection::Forward)
        {
            println!("{}", package.name());
        }
    }
    Ok(())
}

#[span_fn]
pub(crate) fn changed_since_impl<'g>(
    git_cli: &GitCli,
    ctx: &'g Context,
    base: &str,
) -> Result<DeterminatorSet<'g>> {
    let thread_pool = rayon::ThreadPoolBuilder::new()
        .start_handler(|_tid| init_thread_stream())
        .exit_handler(|_tid| flush_thread_buffer())
        .build()
        .unwrap();
    let merge_base = git_cli.merge_base(base)?;

    let (old_graph, (new_graph, files_changed)) = thread_pool.install(|| {
        rayon::join(
            || {
                span_scope!("changed_since_impl::old_graph");
                trace!("building old graph");
                git_cli.package_graph_at(&merge_base).map(|old_graph| {
                    // Initialize the feature graph since it will be required later on.
                    old_graph.feature_graph();
                    old_graph
                })
            },
            || {
                rayon::join(
                    || {
                        span_scope!("changed_since_impl::new_graph");
                        trace!("building new graph");
                        ctx.package_graph().map(|new_graph| {
                            // Initialize the feature graph since it will be required later on.
                            new_graph.feature_graph();
                            new_graph
                        })
                    },
                    || {
                        span_scope!("changed_since_impl::files_changed");
                        // Get the list of files changed between the merge base and the current dir.
                        trace!("getting files changed");
                        git_cli.files_changed_between(&merge_base, None, None)
                    },
                )
            },
        )
    });

    let (old_graph, new_graph, files_changed) = (old_graph?, new_graph?, files_changed?);

    trace!("running determinator");
    span_scope!("running_determinator");
    let mut determinator = Determinator::new(&old_graph, new_graph);
    let mut cargo_options = Determinator::default_cargo_options();
    determinator
        .add_changed_paths(&files_changed)
        .set_cargo_options(cargo_options.set_resolver(CargoResolverVersion::V2))
        .set_rules(&ctx.config().determinator)
        .map_err(|err| Error::new("failed setting the rules").with_source(err))?;

    Ok(determinator.compute())
}
