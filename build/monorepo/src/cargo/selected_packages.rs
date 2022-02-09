// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;

use clap::Args;
use guppy::graph::{BuildTargetId, DependencyDirection};
use lgn_tracing::{span_fn, warn};

use crate::changed_since::changed_since_impl;
use crate::context::Context;
use crate::{Error, Result};

/// Arguments for the Cargo package selector.
#[derive(Debug, Args, Default, Clone)]
pub struct SelectedPackageArgs {
    #[clap(long, short, number_of_values = 1)]
    /// Run on the provided packages
    pub(crate) package: Vec<String>,
    #[clap(long, short, number_of_values = 1)]
    /// Run on the specified members (package subsets)
    pub(crate) members: Vec<String>,
    #[clap(long, number_of_values = 1)]
    /// Exclude packages
    pub(crate) exclude: Vec<String>,
    #[clap(long, short)]
    /// Run on packages changed since the merge base of this commit
    pub(crate) changed_since: Option<String>,
    #[clap(long)]
    /// Valid only with `--changed-since <BASE>`
    pub(crate) direct_only: bool,
    #[clap(long)]
    /// Run on all packages in the workspace
    pub(crate) workspace: bool,
}

impl SelectedPackageArgs {
    #[span_fn]
    pub fn to_selected_packages<'a>(&'a self, ctx: &'a Context) -> Result<SelectedPackages<'a>> {
        // Mutually exclusive options -- only one of these can be provided.
        {
            let mut exclusive = vec![];

            if !self.package.is_empty() {
                exclusive.push("--package");
            } else if !self.members.is_empty() {
                exclusive.push("--members");
            }

            if self.workspace {
                exclusive.push("--workspace");
            }

            if exclusive.len() > 1 {
                let err_msg = exclusive.join(", ");
                return Err(Error::new(format!("can only specify one of {}", err_msg)));
            }
        }

        let mut includes = if self.workspace {
            SelectedInclude::Workspace
        } else if !self.package.is_empty() || !self.members.is_empty() {
            SelectedInclude::includes(
                ctx,
                self.package.iter().map(String::as_str),
                self.members.iter().map(String::as_str),
            )?
        } else {
            SelectedInclude::default_cwd(ctx)?
        };

        // Intersect with --changed-since if specified.
        if let Some(base) = &self.changed_since {
            let git_cli = ctx.git_cli().map_err(|err| {
                err.with_explanation(
                    "May only use --changes-since if working in a local git repository.",
                )
            })?;
            let determinator_set = changed_since_impl(git_cli, ctx, base)?;
            if self.direct_only {
                includes = includes.intersection(
                    determinator_set
                        .path_changed_set
                        .packages(DependencyDirection::Forward)
                        .map(|package| package.name()),
                );
            } else {
                includes = includes.intersection(
                    determinator_set
                        .affected_set
                        .packages(DependencyDirection::Forward)
                        .map(|package| package.name()),
                );
            }
        }

        let mut ret = SelectedPackages::new(includes);

        if !self.exclude.is_empty() {
            let workspace = ctx.package_graph()?.workspace();
            // Check that all the excluded package names are valid.
            let (known, unknown): (Vec<_>, Vec<_>) =
                ctx.partition_workspace_names(self.exclude.iter().map(String::as_str))?;
            if !unknown.is_empty() {
                warn!(
                    "excluded package(s) `{}` not found in workspace `{}`",
                    unknown.join(", "),
                    workspace.root()
                );
            }

            ret.add_excludes(known);
        }

        Ok(ret)
    }
}

/// Package selector for Cargo commands.
///
/// This may represent any of the following:
/// * the entire workspace
/// * a single package without arguments
/// * a list of packages
///
/// This may also exclude a set of packages. Note that currently, excludes only work in the "entire
/// workspace" and "list of packages" situations. They are ignored if a specific local package is
/// being built. (This is an extension on top of Cargo itself, which only supports --exclude
/// together with --workspace.)
///
/// Excludes are applied after includes. This allows changed-since to support excludes, even if only
/// a subset of the workspace changes.
#[derive(Clone, Debug)]
pub struct SelectedPackages<'a> {
    pub(super) includes: SelectedInclude<'a>,
    pub(super) excludes: BTreeSet<&'a str>,
}

impl<'a> SelectedPackages<'a> {
    pub(super) fn new(includes: SelectedInclude<'a>) -> Self {
        Self {
            includes,
            excludes: BTreeSet::new(),
        }
    }

    /// Adds excludes for this `SelectedPackages`.
    pub fn add_excludes(&mut self, exclude_names: impl IntoIterator<Item = &'a str>) -> &mut Self {
        self.excludes.extend(exclude_names);
        self
    }

    // ---
    // Helper methods
    // ---

    pub(super) fn should_invoke(&self) -> bool {
        match &self.includes {
            SelectedInclude::Workspace => true,
            SelectedInclude::Includes(includes) => {
                // If everything in the include set is excluded, a command invocation isn't needed.
                includes.iter().any(|p| !self.excludes.contains(p))
            }
        }
    }

    pub fn includes_package(&self, package: &str) -> bool {
        match &self.includes {
            SelectedInclude::Workspace => true,
            SelectedInclude::Includes(includes) => {
                // If everything in the include set is excluded, a command invocation isn't needed.
                includes
                    .iter()
                    .any(|p| !self.excludes.contains(p) && *p == package)
            }
        }
    }

    /// if the whole workspace is selected, break it down into an includes list
    pub fn flatten_bins(self, ctx: &'a Context) -> Result<Vec<&str>> {
        let workspace = ctx.package_graph()?.workspace();
        match self.includes {
            SelectedInclude::Workspace => Ok(workspace
                .iter()
                .filter_map(|pkg| {
                    if !self.excludes.contains(pkg.name())
                        && pkg.build_targets().any(|build_target| {
                            matches!(build_target.id(), BuildTargetId::Binary(_))
                        })
                    {
                        Some(pkg.name())
                    } else {
                        None
                    }
                })
                .collect()),
            SelectedInclude::Includes(includes) => Ok(includes
                .iter()
                .filter_map(|name| {
                    let pkg = workspace.member_by_name(name);
                    if pkg.is_ok()
                        && !self.excludes.contains(name)
                        && pkg.unwrap().build_targets().any(|build_target| {
                            matches!(build_target.id(), BuildTargetId::Binary(_))
                        })
                    {
                        Some(*name)
                    } else {
                        None
                    }
                })
                .collect()),
        }
    }

    pub fn select_package_from_bin(&mut self, bin: &str, ctx: &'a Context) -> Result<()> {
        match self.includes {
            SelectedInclude::Workspace => {
                let includes = self.includes.intersection(
                    ctx.package_graph()?
                        .workspace()
                        .iter()
                        .filter(|package| {
                            package.build_target(&BuildTargetId::Binary(bin)).is_some()
                        })
                        .map(|package| package.name()),
                );
                if includes.is_empty() {
                    return Err(Error::new(format!(
                        "no package contained a binary named `{}`",
                        bin
                    )));
                }
                self.includes = includes;
            }
            SelectedInclude::Includes(_) => {}
        }
        Ok(())
    }

    pub fn select_package_from_example(&mut self, example: &str, ctx: &'a Context) -> Result<()> {
        match self.includes {
            SelectedInclude::Workspace => {
                let includes = self.includes.intersection(
                    ctx.package_graph()?
                        .workspace()
                        .iter()
                        .filter(|package| {
                            package
                                .build_target(&BuildTargetId::Example(example))
                                .is_some()
                        })
                        .map(|package| package.name()),
                );
                if includes.is_empty() {
                    return Err(Error::new(format!(
                        "no package contained an example named `{}`",
                        example
                    )));
                }
                self.includes = includes;
            }
            SelectedInclude::Includes(_) => {}
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(super) enum SelectedInclude<'a> {
    Workspace,
    Includes(BTreeSet<&'a str>),
}

impl<'a> SelectedInclude<'a> {
    /// Returns a `SelectedInclude` that selects the specified package and subset names.
    #[allow(clippy::unnecessary_wraps, clippy::needless_pass_by_value)]
    pub fn includes(
        _ctx: &'a Context,
        package_names: impl IntoIterator<Item = &'a str>,
        _subsets: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Self> {
        let names: BTreeSet<_> = package_names.into_iter().collect();

        // Don't need to initialize the package graph if no subsets are specified.
        // TODO
        //for name in subsets {
        //    let workspace = ctx.package_graph()?.workspace();
        //    let subsets = ctx.core().subsets()?;
        //
        //    let name = name.as_ref();
        //    // TODO: turn this into a subset in x.toml
        //    let subset = if name == "production" {
        //        subsets.default_members()
        //    } else {
        //        subsets.get(name).ok_or_else(|| {
        //            let known_subsets: Vec<_> = subsets.iter().map(|(name, _)| name).collect();
        //            let help = known_subsets.join(", ");
        //            Error::new(format!(
        //                "unknown subset '{}' (known subsets are: {}, production)",
        //                name, help
        //            ))
        //        })?
        //    };
        //    let selected = workspace.iter().filter_map(|package| {
        //        if subset.status_of(package.id()) != WorkspaceStatus::Absent {
        //            Some(package.name())
        //        } else {
        //            None
        //        }
        //    });
        //
        //    names.extend(selected);
        //}

        Ok(SelectedInclude::Includes(names))
    }

    /// Returns a `SelectedInclude` that selects the default set of packages for the current
    /// working directory. This may either be the entire workspace or a set of packages inside the
    /// workspace.
    pub fn default_cwd(ctx: &'a Context) -> Result<Self> {
        if ctx.current_dir_is_root() {
            Ok(SelectedInclude::Workspace)
        } else {
            // Select all packages that begin with the current rel dir.
            let rel = ctx.current_rel_dir();
            let workspace = ctx.package_graph()?.workspace();
            let selected = workspace.iter_by_path().filter_map(|(path, package)| {
                // If we're in devtools, run tests for all packages inside devtools.
                // If we're in devtools/x/src, run tests for devtools/x.
                if path.starts_with(rel) || rel.starts_with(path) {
                    Some(package.name())
                } else {
                    None
                }
            });
            Ok(SelectedInclude::Includes(selected.collect()))
        }
    }

    /// Intersects this `SelectedInclude` with the given names.
    pub fn intersection(&self, names: impl IntoIterator<Item = &'a str>) -> Self {
        let names = names.into_iter().collect();
        match self {
            SelectedInclude::Workspace => SelectedInclude::Includes(names),
            SelectedInclude::Includes(includes) => {
                SelectedInclude::Includes(includes.intersection(&names).copied().collect())
            }
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            SelectedInclude::Workspace => false,
            SelectedInclude::Includes(includes) => includes.is_empty(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_should_invoke() {
        let packages = SelectedPackages::new(SelectedInclude::Workspace);
        assert!(packages.should_invoke(), "workspace => invoke");

        let mut packages = SelectedPackages::new(SelectedInclude::Includes(
            vec!["foo", "bar"].into_iter().collect(),
        ));
        packages.add_excludes(vec!["foo"]);
        assert!(packages.should_invoke(), "non-empty packages => invoke");

        let packages = SelectedPackages::new(SelectedInclude::Includes(BTreeSet::new()));
        assert!(!packages.should_invoke(), "no packages => do not invoke");

        let mut packages = SelectedPackages::new(SelectedInclude::Includes(
            vec!["foo", "bar"].into_iter().collect(),
        ));
        packages.add_excludes(vec!["foo", "bar"]);
        assert!(
            !packages.should_invoke(),
            "all packages excluded => do not invoke"
        );
    }
}
