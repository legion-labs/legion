use std::{collections::HashMap, fs};

use camino::Utf8Path;
use determinator::rules::DeterminatorRules;
use lgn_tracing::trace_function;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct MonorepoConfig {
    pub cargo_config: CargoConfig,
    pub determinator: DeterminatorRules,
    pub clippy: Clippy,
    pub rustdoc: RustDoc,
    pub cargo_installs: HashMap<String, CargoInstallation>,
    pub dependencies: Dependencies,
    pub crate_attributes: CrateAttributes,
}

impl MonorepoConfig {
    #[trace_function]
    pub fn new(root: &Utf8Path) -> Result<Self> {
        let monorepo_file = root.join("monorepo.toml");
        let contents = fs::read(&monorepo_file).map_err(|err| {
            Error::new(format!("could not read config file {}", &monorepo_file)).with_source(err)
        })?;
        let mut config: Self = toml::from_slice(&contents).map_err(|err| {
            Error::new(format!("failed to parse config file {}", &monorepo_file)).with_source(err)
        })?;
        if let Some(sscache) = &config.cargo_config.sccache {
            config
                .cargo_installs
                .insert("sccache".to_owned(), sscache.installer.clone());
        }
        Ok(config)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct CargoConfig {
    pub sccache: Option<Sccache>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct CargoInstalls {
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

/// Dependencies lints configurations
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Dependencies {
    pub bans: Vec<DependencyBan>,
}

/// Additional dependencies
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct DependencyBan {
    pub name: String,
    pub version: String,
    pub suggestion: String,
    pub exceptions: Option<Vec<String>>,
}

/// Crate attributes lints configurations
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct CrateAttributes {
    pub name: NameAttribute,
    pub license: LicenseAttribute,
    pub edition: String,
}

/// Name attribute lint configurations
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct NameAttribute {
    name_pattern: String,
    globs: Vec<String>,
}

/// License attribute lint configurations
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct LicenseAttribute {
    spdx: String,
    globs: Vec<String>,
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
pub struct RustDoc {
    pub entry_point: String,
    pub allow: Vec<String>,
    pub warn: Vec<String>,
    pub deny: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct MonorepoConfigContainer {
    pub(crate) monorepo: MonorepoConfig,
}
