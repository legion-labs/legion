use std::fs;

use camino::Utf8Path;
use determinator::rules::DeterminatorRules;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct MonorepoConfig {
    pub cargo_config: CargoConfig,
    pub determinator: DeterminatorRules,
    pub clippy: Clippy,
    pub grcov: CargoTool,
}

impl MonorepoConfig {
    pub fn new(root: &Utf8Path) -> Result<Self> {
        let monorepo_file = root.join("monorepo.toml");
        let contents = fs::read(&monorepo_file).map_err(|err| {
            Error::new(format!("could not read config file {}", &monorepo_file)).with_source(err)
        })?;
        toml::from_slice(&contents).map_err(|err| {
            Error::new(format!("failed to parse config file {}", &monorepo_file)).with_source(err)
        })
    }

    pub fn tools(&self) -> Vec<(String, CargoInstallation)> {
        let mut tools = vec![("grcov".to_owned(), self.grcov.installer.clone())];
        if let Some(sccache) = &self.cargo_config.sccache {
            tools.push(("sccache".to_owned(), sccache.installer.clone()));
        }
        tools
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct PackageConfig {}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct CargoConfig {
    pub sccache: Option<Sccache>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct CargoTool {
    pub installer: CargoInstallation,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Sccache {
    /// Sccache Url
    pub installer: CargoInstallation,
    /// Where cargo home must reside for sccache to function properly.  Paths are embedded in binaries by rustc.
    pub required_cargo_home: String,
    /// Where the git repo for this project must reside.  Paths are embedded in binaries by rustc.
    pub required_git_home: String,
    /// s3 bucket location
    pub bucket: String,
    /// prefix to files uploaded in to s3
    pub prefix: Option<String>,
    /// utility of this seems to change in stable/vs rusoto, left for completeness.
    pub endpoint: Option<String>,
    /// AWS region of the bucket if necessary.
    pub region: Option<String>,
    /// Only used in stable, sscache delete when rusoto is merged in
    pub ssl: Option<bool>,
    /// If the bucket is public
    pub public: Option<bool>,
    /// Extra environment variables to set for the sccache server.
    pub envs: Option<Vec<(String, String)>>,
}

///
/// These can be passed to the installer.rs, which can check the installation against the version number supplied,
/// or install the cargo tool via either githash/repo if provided or with simply the version if the artifact is released
/// to crates.io.
///
/// Unfortunately there is no gaurantee that the installation is correct if the version numbers match as the githash
/// is not stored by default in the version number.
///
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct CargoInstallation {
    /// The version string that must match the installation, otherwise a fresh installation will occure.
    pub version: String,
    /// Overrides the default install with a specific git repo. git-rev is required.
    pub git: Option<String>,
    /// only used if the git url is set.  This is the full git hash.
    pub git_rev: Option<String>,
    /// features to enable in the installation.
    pub features: Option<Vec<String>>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Clippy {
    pub allow: Vec<String>,
    pub warn: Vec<String>,
    pub deny: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct MonorepoConfigContainer {
    pub(crate) monorepo: MonorepoConfig,
}
