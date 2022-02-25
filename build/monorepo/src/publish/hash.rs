use std::{collections::HashMap, fs::read_dir};

use camino::{Utf8Path, Utf8PathBuf};
use guppy::{graph::PackageMetadata, PackageId};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::metadata::PublishMetadata;
use crate::{Error, Result};

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
    sources: String,
    dist_metadatas: Option<&'g Vec<PublishMetadata>>,
}

impl<'g> HashSource<'g> {
    pub(super) fn hash(
        package: &PackageMetadata<'g>,
        hash_cache: &mut HashMap<PackageId, String>,
        dist_metadatas: Option<&'g Vec<PublishMetadata>>,
    ) -> Result<String> {
        if let Some(hash) = hash_cache.get(package.id()) {
            return Ok(hash.clone());
        }

        let direct_links = package
            .direct_links()
            .map(|link| {
                let link_package = link.to();
                if link_package.in_workspace() {
                    Self::hash(&link_package, hash_cache, None)
                } else {
                    Ok(link_package.id().to_string())
                }
            })
            .collect::<Result<Vec<_>>>()?;

        let hash_source = Self {
            name: package.name(),
            version: package.version(),
            authors: package.authors(),
            description: package.description(),
            license: package.license(),
            license_file: package.license_file(),
            categories: package.categories(),
            keywords: package.keywords(),
            readme: package.readme(),
            repository: package.repository(),
            edition: package.edition(),
            links: package.links(),
            direct_links,
            sources: Self::compute_hash(package.manifest_path().parent().unwrap())?,
            dist_metadatas,
        };
        let mut state = Sha256::new();

        // There is no reason for this write to ever fail so unwrap is fine.
        serde_json::to_writer(&mut state, &hash_source).unwrap();
        let hash = format!("sha256:{:x}", state.finalize());
        hash_cache.insert(package.id().clone(), hash.clone());
        Ok(hash)
    }

    fn compute_hash(root: &Utf8Path) -> Result<String> {
        let mut files = vec![];
        Self::visit_crate_files(root, &mut files)
            .map_err(|e| Error::new("failed to list crate files").with_source(e))?;
        files.sort();

        let mut state = Sha256::new();
        for file in files {
            let content = std::fs::read(file)
                .map_err(|err| Error::new("failed to read file").with_source(err))?;
            state.update(&content);
        }
        Ok(format!("{:x}", state.finalize()))
    }

    fn visit_crate_files(dir: &Utf8Path, files: &mut Vec<Utf8PathBuf>) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    Self::visit_crate_files(path.to_string_lossy().as_ref().into(), files)?;
                } else if !Self::filter_path(&path) {
                    files.push(path.to_string_lossy().as_ref().into());
                }
            }
        }
        Ok(())
    }

    fn filter_path(path: &std::path::Path) -> bool {
        if path.is_symlink() {
            return true;
        }
        let path_str = path.to_string_lossy();
        if path_str.contains("node_modules") || path_str.ends_with("Cargo.toml") {
            return true;
        }

        false
    }
}
