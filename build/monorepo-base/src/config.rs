use std::{collections::HashMap, fs};

use camino::Utf8Path;
use serde::Deserialize;

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct MonorepoBaseConfig {
    pub cargo: Cargo,
}

pub const MONOREPO_FILE: &str = "monorepo.toml";
pub const MONOREPO_DEPTH: usize = 2;

impl MonorepoBaseConfig {
    /// Reads the monorepo config file from the given root directory.
    ///
    /// # Errors
    /// If the config file cannot be parsed, an error is returned.
    ///
    pub fn new(root: &Utf8Path) -> Result<Self, toml::de::Error> {
        let monorepo_file = root.join(MONOREPO_FILE);
        toml::from_slice(&fs::read(monorepo_file).unwrap())
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Cargo {
    pub sccache: Option<Sccache>,
    pub user_sccache: Option<Sccache>,
    pub installs: HashMap<String, CargoInstallation>,
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Sccache {
    /// Where cargo home must reside for sccache to function properly.  Paths are embedded in binaries by rustc.
    pub required_cargo_home: HostConfig,
    /// Where the git repo for this project must reside.  Paths are embedded in binaries by rustc.
    pub required_git_home: HostConfig,
    /// s3 bucket location
    pub bucket: String,
    /// prefix to files uploaded in to s3
    pub prefix: Option<String>,
    /// utility of this seems to change in stable/vs rusoto, left for completeness.
    pub endpoint: Option<String>,
    /// AWS region of the bucket if necessary.
    pub region: Option<String>,
    /// Only used in stable, sccache delete when rusoto is merged in
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
#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
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

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", untagged)]
pub enum HostConfig {
    Default(String),
    Platform {
        windows: String,
        linux: String,
        macos: String,
    },
}

impl HostConfig {
    pub fn as_str(&self) -> &str {
        match self {
            HostConfig::Default(s) => s.as_str(),
            HostConfig::Platform {
                windows,
                linux,
                macos,
            } => {
                if cfg!(windows) {
                    windows.as_str()
                } else if cfg!(target_os = "linux") {
                    linux.as_str()
                } else {
                    macos.as_str()
                }
            }
        }
    }
}

impl Default for HostConfig {
    fn default() -> Self {
        Self::Default("lldb".to_string())
    }
}
