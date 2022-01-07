use std::collections::BTreeMap;

use camino::Utf8Path;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{metadata::DistTargetMetadata, sources::Sources, Package, Result};

/// A structure whose sole purpose is to help compute a deterministic hash of a
/// given package.
#[derive(Serialize)]
pub(crate) struct HashSource<'g> {
    name: &'g str,
    version: &'g semver::Version,
    authors: &'g [String],
    description: Option<&'g str>,
    license: Option<&'g str>,
    license_file: Option<&'g Utf8Path>,
    categories: &'g [String],
    keywords: &'g [String],
    readme: Option<&'g Utf8Path>,
    repository: Option<&'g str>,
    edition: &'g str,
    links: Option<&'g str>,
    direct_links: Vec<String>,
    sources: &'g Sources,
    dist_targets: &'g BTreeMap<String, DistTargetMetadata>,
}

impl<'g> HashSource<'g> {
    pub(crate) fn new(package: &'g Package<'g>) -> Result<Self> {
        let direct_links = package
            .package_metadata()
            .direct_links()
            .map(|link| {
                let link_package = link.to();

                // If the package we depend on is a package from the workspace,
                // we actually depend on its hash instead of its id so that we
                // cover all cases of that package changing.
                if link_package.in_workspace() {
                    package
                        .context()
                        .resolve_package_by_name(link_package.name())?
                        .hash()
                } else {
                    Ok(link_package.id().to_string())
                }
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            name: package.package_metadata().name(),
            version: package.package_metadata().version(),
            authors: package.package_metadata().authors(),
            description: package.package_metadata().description(),
            license: package.package_metadata().license(),
            license_file: package.package_metadata().license_file(),
            categories: package.package_metadata().categories(),
            keywords: package.package_metadata().keywords(),
            readme: package.package_metadata().readme(),
            repository: package.package_metadata().repository(),
            edition: package.package_metadata().edition(),
            links: package.package_metadata().links(),
            direct_links,
            sources: package.sources(),
            dist_targets: &package.monorepo_metadata().dist_targets,
        })
    }

    pub(crate) fn hash(&self) -> String {
        let mut state = Sha256::new();

        // There is no reason for this write to ever fail so unwrap is fine.
        serde_json::to_writer(&mut state, &self).unwrap();

        format!("sha256:{:x}", state.finalize())
    }
}
