use std::{collections::HashMap, fs};

use camino::Utf8Path;
use determinator::rules::DeterminatorRules;
use lgn_tracing::span_fn;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct MonorepoConfig {
    pub cargo: Cargo,
    pub vscode: VsCode,
    pub clippy: Clippy,
    pub rustdoc: RustDoc,
    pub lints: Lints,
    pub determinator: DeterminatorRules,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct LocalMonorepoConfig {
    pub vscode: Option<VsCode>,
    pub cargo: Option<Cargo>,
}

impl MonorepoConfig {
    #[span_fn]
    pub fn new(root: &Utf8Path) -> Result<Self> {
        let mut contents = vec![];
        let monorepo_file = root.join("monorepo.toml");
        let mut config: Self = read_config(&monorepo_file, &mut contents)?;
        let local_monorepo_file = root.join("monorepo.local.toml");
        if std::fs::metadata(&local_monorepo_file).is_ok() {
            let local_config = read_config(&local_monorepo_file, &mut contents)?;
            config.merge_with(&local_config);
        }
        Ok(config)
    }
    pub fn merge_with(&mut self, other: &LocalMonorepoConfig) {
        if let Some(vs_code) = &other.vscode {
            self.vscode = vs_code.clone();
        }
        if let Some(cargo) = &other.cargo {
            let installs = self.cargo.installs.clone();
            self.cargo = cargo.clone();
            // should probably invert this, overrides taking precedence ?
            self.cargo.installs.extend(installs.into_iter());
        }
    }
}

pub fn read_config<'de, T: Deserialize<'de>>(
    path: &Utf8Path,
    contents: &'de mut Vec<u8>,
) -> Result<T> {
    *contents = fs::read(path).map_err(|err| {
        Error::new(format!("could not read config file {}", path)).with_source(err)
    })?;
    let config: T = toml::from_slice(contents).map_err(|err| {
        Error::new(format!("failed to parse config file {}", path)).with_source(err)
    })?;
    Ok(config)
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Cargo {
    pub sccache: Option<Sccache>,
    pub user_sccache: Option<Sccache>,
    pub installs: HashMap<String, CargoInstallation>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
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

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Lints {
    pub direct_dependencies: DirectDependencies,
    pub crate_attributes: CrateAttributes,
}

/// Dependencies lints configurations
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct DirectDependencies {
    pub bans: Vec<DependencyBan>,
}

/// Additional dependencies
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct DependencyBan {
    pub name: String,
    pub version: String,
    pub reason: String,
    pub exceptions: Option<Vec<String>>,
}

/// Crate attributes lints configurations
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct CrateAttributes {
    pub name_rules: Vec<RegexRule>,
    pub bins_rules: Vec<RegexRule>,
    pub license_rules: Vec<LicenseAttribute>,
    pub edition: String,
}

/// Name attribute lint configurations
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct RegexRule {
    pub help: String,
    pub pattern: String,
    pub negative_pattern: Option<String>,
    pub globs: Vec<String>,
    pub glob_literal_separator: Option<bool>,
}

/// License attribute lint configurations
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct LicenseAttribute {
    pub spdx: String,
    pub globs: Vec<String>,
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

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct VsCode {
    pub debugger_type: HostConfig,
    pub compounds: HashMap<String, Vec<String>>,
    pub overrides: HashMap<String, HashMap<String, Vec<String>>>,
    pub disable_prelaunch: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
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