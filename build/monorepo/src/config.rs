use camino::Utf8Path;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct MonorepoConfig {
    pub(crate) cargo_config: CargoConfig,
}

impl MonorepoConfig {
    pub fn new(root: &Utf8Path) -> Result<Self> {
        let manifest = cargo_toml::Manifest::<MonorepoConfigContainer>::from_path_with_metadata(
            root.join("Cargo.toml"),
        )
        .map_err(|err| {
            Error::new(format!("failed to parse workspace manifest {}", err)).with_source(err)
        })?;

        let workspace = manifest
            .workspace
            .ok_or_else(|| Error::new("manifest is not a workspace"))?;

        // Serialize will fail if th metadata is missing
        Ok(workspace.metadata.unwrap().monorepo)
    }
    pub(crate) fn cargo_config(&self) -> &CargoConfig {
        &self.cargo_config
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct PackageConfig {}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct CargoConfig {
    pub sccache: Option<bool>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct MonorepoConfigContainer {
    pub(crate) monorepo: MonorepoConfig,
}
