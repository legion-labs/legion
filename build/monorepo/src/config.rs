use std::{collections::HashMap, fs};

use camino::Utf8Path;
use determinator::rules::DeterminatorRules;
use lgn_tracing::span_fn;
use monorepo_base::config::{HostConfig, Sccache, Tools, MONOREPO_CONFIG_PATH};
use serde::Deserialize;

use crate::{Error, Result};
#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct MonorepoConfig {
    pub tools: Tools,
    pub sccache: Sccache,
    pub lints: Lints,
    pub editor: Editor,
    pub publish: Publish,
    pub determinator: DeterminatorRules,
    pub npm: Npm,
    pub package_sets: PackageSets,
}

impl MonorepoConfig {
    #[span_fn]
    pub fn new(root: &Utf8Path) -> Result<Self> {
        let tools = Tools::new(root)
            .map_err(|err| Error::new("failed to parse tools file").with_source(err))?;
        let sccache = Sccache::new(root)
            .map_err(|err| Error::new("failed to parse sccache file").with_source(err))?;
        let lints = Lints::new(root)?;
        let editor = Editor::new(root)?;
        let publish = Publish::new(root)?;
        let determinator = {
            let determinator_file = root.join(MONOREPO_CONFIG_PATH).join("determinator.toml");
            toml::from_slice(&fs::read(&determinator_file).unwrap()).map_err(|err| {
                Error::new(format!("could not read config file {}", determinator_file))
                    .with_source(err)
            })?
        };
        let npm = Npm::new(root)?;
        let package_sets = PackageSets::new(root)?;

        Ok(Self {
            tools,
            sccache,
            lints,
            editor,
            publish,
            determinator,
            npm,
            package_sets,
        })
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Lints {
    pub direct_dependencies: DirectDependencies,
    pub crate_attributes: CrateAttributes,
    pub clippy: Clippy,
    pub rustdoc: RustDoc,
}

impl Lints {
    pub fn new(root: &Utf8Path) -> Result<Self> {
        let lints_file = root.join(MONOREPO_CONFIG_PATH).join("lints.toml");
        toml::from_slice(&fs::read(&lints_file).unwrap()).map_err(|err| {
            Error::new(format!("could not read config file {}", lints_file)).with_source(err)
        })
    }
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
    pub allow: Vec<String>,
    pub warn: Vec<String>,
    pub deny: Vec<String>,
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Editor {
    pub vscode: VsCode,
}

impl Editor {
    pub fn new(root: &Utf8Path) -> Result<Self> {
        let lints_file = root.join(MONOREPO_CONFIG_PATH).join("editor.toml");
        toml::from_slice(&fs::read(&lints_file).unwrap()).map_err(|err| {
            Error::new(format!("could not read config file {}", lints_file)).with_source(err)
        })
    }
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
pub struct Publish {
    pub s3: S3Bucket,
    pub rustdoc: RustdocPublish,
}

impl Publish {
    pub fn new(root: &Utf8Path) -> Result<Self> {
        let publish_file = root.join(MONOREPO_CONFIG_PATH).join("publish.toml");
        toml::from_slice(&fs::read(&publish_file).unwrap()).map_err(|err| {
            Error::new(format!("could not read config file {}", publish_file)).with_source(err)
        })
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct S3Bucket {
    /// s3 bucket location
    pub bucket: String,
    /// prefix to files uploaded in to s3
    pub prefix: String,
    /// AWS region of the bucket if necessary.
    pub region: Option<String>,
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct RustdocPublish {
    pub entry_point: String,
}

#[derive(Clone, Default, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Npm {
    pub package_manager: String,
    #[serde(default)]
    pub include: Vec<String>,
}

impl Npm {
    pub fn new(root: &Utf8Path) -> Result<Self> {
        let npm_file = root.join(MONOREPO_CONFIG_PATH).join("npm.toml");
        toml::from_slice(&fs::read(&npm_file).unwrap()).map_err(|err| {
            Error::new(format!("could not read config file {}", npm_file)).with_source(err)
        })
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct PackageSets {
    pub package_sets: HashMap<String, Vec<String>>,
}

impl PackageSets {
    pub fn new(root: &Utf8Path) -> Result<Self> {
        let package_sets_file = root.join(MONOREPO_CONFIG_PATH).join("pkg_sets.toml");
        toml::from_slice(&fs::read(&package_sets_file).unwrap()).map_err(|err| {
            Error::new(format!("could not read config file {}", package_sets_file)).with_source(err)
        })
    }
}
