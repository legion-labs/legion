use determinator::rules::PathMatch;
use determinator::Determinator;
use lgn_tracing::{span_fn, span_scope};

use crate::context::Context;
use crate::{action_step, Error, Result};

#[span_fn]
pub fn run(ctx: &Context) -> Result<()> {
    action_step!("Monorepo", "Running rules determination");
    let git_cli = ctx.git_cli().map_err(|err| {
        err.with_explanation("changed-since` must be run within a project cloned from a git repo.")
    })?;
    let tracked_files = git_cli.tracked_files()?;
    let graph = ctx.package_graph().map(|new_graph| {
        span_scope!("rules_coverage::feature_graph");
        // Initialize the feature graph since it will be required later on.
        new_graph.feature_graph();
        new_graph
    })?;

    span_scope!("rules_coverage::match_paths");
    // we can use the same graph since match path actually does not use the old graph
    let mut determinator = Determinator::new(graph, graph);
    determinator
        .set_rules(&ctx.config().determinator)
        .map_err(|err| Error::new("failed setting the rules").with_source(err))?;
    let mut file_not_matched = false;
    for tracked_file in tracked_files {
        if determinator.match_path(tracked_file, |_| ()) == PathMatch::NoMatches {
            println!("    ---> unmatched file:  {}", tracked_file);
            file_not_matched = true;
        }
    }
    if file_not_matched {
        Err(Error::new("found not matched files"))
    } else {
        Ok(())
    }
}
