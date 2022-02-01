use lgn_tracing::span_fn;
use monorepo_base::action_step;
use semver::VersionReq;

use crate::{context::Context, Error, Result};

#[span_fn]
pub fn run(ctx: &Context) -> Result<()> {
    action_step!("Lint", "Running direct dependencies checks");
    let workspace = ctx.package_graph()?.workspace();
    let bans: Vec<_> = ctx
        .config()
        .lints
        .direct_dependencies
        .bans
        .iter()
        .map(|dep| {
            (
                dep.name.as_str(),
                VersionReq::parse(&dep.version),
                dep.reason.clone(),
                dep.exceptions
                    .as_ref()
                    .map_or(&[] as &[String], Vec::as_slice),
            )
        })
        .collect();
    // validate bans
    for (name, version, _, exceptions) in &bans {
        if version.is_err() {
            return Err(Error::new(format!(
                "invalid version requirement for ban {}: {}",
                name,
                version.as_ref().err().unwrap(),
            )));
        }
        for exception in *exceptions {
            if !workspace.contains_name(exception) {
                return Err(Error::new(format!(
                    "exception {} is not in the workspace",
                    exception
                )));
            }
        }
    }
    for package in workspace.iter() {
        for plink in package.direct_links() {
            for (name, version, reason, exceptions) in &bans {
                let dep = plink.to();
                if *name == dep.name()
                    && version.as_ref().unwrap().matches(dep.version())
                    && exceptions.iter().all(|s| s != package.name())
                {
                    return Err(Error::new(format!(
                        "package {}@{} is banned, but is depended on by {}, please use {}",
                        dep.name(),
                        dep.version(),
                        package.name(),
                        reason,
                    )));
                }
            }
        }
    }
    Ok(())
}
