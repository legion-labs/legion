use std::collections::BTreeSet;

use clap::Args;

/// Arguments for the Cargo package selector.
#[derive(Debug, Args)]
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
    changed_since: Option<String>,
    #[clap(long)]
    /// Run on all packages in the workspace
    pub(crate) workspace: bool,
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
    //TODO
    //pub(super) includes: SelectedInclude<'a>,
    //pub(super) excludes: BTreeSet<&'a str>,
    pub excludes: BTreeSet<&'a str>,
}
