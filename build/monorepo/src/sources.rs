use std::{
    collections::BTreeMap,
    iter::once,
    path::{Path, PathBuf},
};

use cargo::core::Source;
use serde::Serialize;

use crate::{context::Context, Error, Result};

/// Represent the sources files for a package.
///
/// This structure does not only contain the rust source files but any file that
/// belong to - and that can possibly be used by - the package.
///
/// As an exception, the manifest file is never included in this structure.
#[derive(Debug, Clone, Serialize)]
pub struct Sources(BTreeMap<PathBuf, Vec<u8>>);

impl Sources {
    pub fn from_package(
        context: &Context,
        package: &guppy::graph::PackageMetadata<'_>,
    ) -> Result<Self> {
        let workspace = &context.workspace()?;
        let core_package = workspace
            .members()
            .find(|pkg| pkg.name().as_str() == package.name())
            .ok_or_else(|| {
                Error::new("failed to find package").with_explanation(format!(
                    "Could not find a package named `{}` in the current workspace.",
                    package.name()
                ))
            })?;

        Self::new(workspace, core_package)
    }

    fn new(workspace: &cargo::core::Workspace<'_>, pkg: &cargo::core::Package) -> Result<Self> {
        let mut path_source = cargo::sources::PathSource::new(
            pkg.root(),
            pkg.package_id().source_id(),
            workspace.config(),
        );

        path_source
            .update()
            .map_err(|err| Error::new("failed to update path source").with_source(err))?;

        Ok(Self(
            path_source
                .list_files(pkg)
                .map_err(|err| Error::new("failed to list files").with_source(err))?
                .into_iter()
                .chain(once(pkg.manifest_path().to_path_buf()))
                .filter_map(|path| {
                    (path != pkg.manifest_path()).then(|| Self::read_generic_file(path))
                })
                .collect::<Result<Vec<(PathBuf, Vec<u8>)>>>()?
                .into_iter()
                .collect(),
        ))
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.0.contains_key(path)
    }

    pub fn read_generic_file(path: PathBuf) -> Result<(PathBuf, Vec<u8>)> {
        std::fs::read(&path)
            .map(|bytes| (path, bytes))
            .map_err(|err| Error::new("failed to read file").with_source(err))
    }
}
