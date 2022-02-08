use std::{collections::HashMap, fs};

use camino::Utf8Path;
use determinator::rules::DeterminatorRules;
use lgn_tracing::span_fn;
use monorepo_base::config::{Cargo, HostConfig, MONOREPO_FILE};
use serde::Deserialize;

use crate::{Error, Result};
#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct MonorepoConfig {
    pub cargo: Cargo,
    pub vscode: VsCode,
    pub clippy: Clippy,
    pub rustdoc: RustDoc,
    pub lints: Lints,
    pub dist: Dist,
    pub determinator: DeterminatorRules,
    pub npm: Npm,
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct LocalMonorepoConfig {
    pub vscode: Option<VsCode>,
    pub cargo: Option<Cargo>,
}

impl MonorepoConfig {
    #[span_fn]
    pub fn new(root: &Utf8Path) -> Result<Self> {
        let mut contents = vec![];
        let monorepo_file = root.join(MONOREPO_FILE);
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

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Lints {
    pub direct_dependencies: DirectDependencies,
    pub crate_attributes: CrateAttributes,
}

/// Dependencies lints configurations
#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct DirectDependencies {
    pub bans: Vec<DependencyBan>,
}

/// Additional dependencies
#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct DependencyBan {
    pub name: String,
    pub version: String,
    pub reason: String,
    pub exceptions: Option<Vec<String>>,
}

/// Crate attributes lints configurations
#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct CrateAttributes {
    pub name_rules: Vec<RegexRule>,
    pub bins_rules: Vec<RegexRule>,
    pub license_rules: Vec<LicenseAttribute>,
    pub edition: String,
}

/// Name attribute lint configurations
#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct RegexRule {
    pub help: String,
    pub pattern: String,
    pub negative_pattern: Option<String>,
    pub globs: Vec<String>,
    pub glob_literal_separator: Option<bool>,
}

/// License attribute lint configurations
#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct LicenseAttribute {
    pub spdx: String,
    pub globs: Vec<String>,
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Clippy {
    pub allow: Vec<String>,
    pub warn: Vec<String>,
    pub deny: Vec<String>,
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct RustDoc {
    pub entry_point: String,
    pub allow: Vec<String>,
    pub warn: Vec<String>,
    pub deny: Vec<String>,
}

#[derive(Default, Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct VsCode {
    pub debugger_type: HostConfig,
    pub compounds: HashMap<String, Vec<String>>,
    pub overrides: HashMap<String, HashMap<String, Vec<String>>>,
    pub disable_prelaunch: bool,
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Dist {
    /// s3 bucket location
    pub bucket: String,
    /// prefix to files uploaded in to s3
    pub prefix: String,
    /// AWS region of the bucket if necessary.
    pub region: Option<String>,
}

#[derive(Clone, Default, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Npm {
    pub package_manager: String,
    pub build_script: String,
    pub clean_script: String,
    pub check_script: String,
    pub format_script: String,
    pub test_script: String,
    #[serde(default)]
    pub excluded_dirs: Vec<String>,
}
