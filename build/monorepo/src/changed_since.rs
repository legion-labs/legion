// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::git::GitCli;
use crate::{context::Context, Error, Result};
use determinator::Determinator;
use guppy::graph::{DependencyDirection, PackageSet};
use lgn_telemetry::trace;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// List packages changed since this commit
    pub(crate) base: String,
}

pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let git_cli = ctx.git_cli().map_err(|err| {
        err.with_explanation("changed-since` must be run within a project cloned from a git repo.")
    })?;
    let affected_set = changed_since_impl(git_cli, ctx, &args.base)?;
    for package in affected_set.packages(DependencyDirection::Forward) {
        println!("{}", package.name());
    }
    Ok(())
}

pub(crate) fn changed_since_impl<'g>(
    git_cli: &GitCli,
    ctx: &'g Context,
    base: &str,
) -> Result<PackageSet<'g>> {
    let merge_base = git_cli.merge_base(base)?;
    let (old_graph, (new_graph, files_changed)) = rayon::join(
        || {
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
                    trace!("building new graph");
                    ctx.package_graph().map(|new_graph| {
                        // Initialize the feature graph since it will be required later on.
                        new_graph.feature_graph();
                        new_graph
                    })
                },
                || {
                    // Get the list of files changed between the merge base and the current dir.
                    trace!("getting files changed");
                    git_cli.files_changed_between(&merge_base, None, None)
                },
            )
        },
    );
    let (old_graph, new_graph, files_changed) = (old_graph?, new_graph?, files_changed?);

    trace!("running determinator");
    let mut determinator = Determinator::new(&old_graph, new_graph);
    determinator
        .add_changed_paths(&files_changed)
        .set_rules(&ctx.config().determinator)
        .map_err(|err| Error::new("failed setting the rules").with_source(err))?;

    let determinator_set = determinator.compute();
    Ok(determinator_set.affected_set)
}
