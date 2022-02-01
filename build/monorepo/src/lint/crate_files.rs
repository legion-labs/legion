use lgn_tracing::span_fn;
use monorepo_base::action_step;

use crate::{context::Context, Error, Result};

#[span_fn]
pub fn run(ctx: &Context) -> Result<()> {
    action_step!("Lint", "Running crate file checks");
    let workspace = ctx.package_graph()?.workspace();
    let mut lock_file_rules = 0_u32;
    for (path, package) in workspace.iter_by_path() {
        // check if package has a Cargo.lock file
        let mut path = path.to_path_buf();
        path.push("Cargo.lock");
        if path.exists() {
            eprintln!(
                "{}: should not have a local Cargo.lock file",
                package.name()
            );
            lock_file_rules += 1;
        }
    }
    if lock_file_rules != 0 {
        return Err(Error::new(format!(
            "failed {} lock file rule(s)",
            lock_file_rules
        )));
    }
    Ok(())
}
