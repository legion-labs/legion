use std::{
    borrow::Cow,
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    process::Command,
};

use camino::Utf8PathBuf;
use monorepo_base::{action_step, error_step};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use walkdir::{DirEntry, WalkDir};
use which::which;

use crate::{cargo::SelectedPackages, context::Context, error::Error, Result};

const PACKAGE_JSON: &str = "package.json";

const INSTALL_SCRIPT: &str = "install";

#[derive(Debug, Deserialize)]
struct NpmMetadata {
    name: String,
    path: Utf8PathBuf,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    npm: NpmMetadata,
}

#[derive(Debug, Deserialize)]
struct PackageJson {
    name: String,
    scripts: HashMap<String, String>,
}

impl PackageJson {
    /// Takes a string-like value and return true if it's "package.json"
    fn is_package_json<'a, F: PartialEq<&'a str>>(file_name: F) -> bool {
        file_name == PACKAGE_JSON
    }
}

#[derive(Debug)]
pub struct NpmPackage {
    /// The path of the npm package
    path: PathBuf,
    /// The package json struct
    package_json: PackageJson,
}

impl NpmPackage {
    fn new<'a, P: Into<Cow<'a, Path>>>(path: P, package_json: PackageJson) -> Self {
        Self {
            path: path.into().into_owned(),
            package_json,
        }
    }

    /// Initialize an [`NpmPackage`] from a path
    pub fn from_path<'a, P: Into<Cow<'a, Path>>>(path: P) -> Result<Self> {
        let path = path.into();

        let file = File::open(path.join(PACKAGE_JSON)).map_err(|error| {
            Error::new(format!("Couldn't open {}", path.to_string_lossy())).with_source(error)
        })?;

        let reader = BufReader::new(file);

        let package_json = serde_json::from_reader::<_, PackageJson>(reader).map_err(|error| {
            Error::new(format!(
                r#"Invalid {} file at "{}""#,
                PACKAGE_JSON,
                path.to_string_lossy()
            ))
            .with_source(error)
        })?;

        Ok(Self::new(path, package_json))
    }

    pub fn run_install(&self, config: &NpmWorkspaceConfig) -> Result<()> {
        action_step!(
            "Npm Install",
            "{} ({})",
            self.package_json.name,
            self.path.to_string_lossy()
        );

        let mut cmd = Command::new(&config.package_manager_path);

        let cmd = cmd.arg(INSTALL_SCRIPT).current_dir(&self.path);

        match cmd.output() {
            Ok(output) if output.status.success() => {
                action_step!("Finished", "{}", self.package_json.name)
            }
            Ok(output) => error_step!(
                "Npm Install",
                r#"Couldn't install dependencies for "{}": {}"#,
                self.package_json.name,
                // It's not a typo, it seems some package managers
                // use the stdout channel when an error occurs
                String::from_utf8(output.stdout).unwrap()
            ),
            Err(error) => error_step!(
                "Npm Install",
                r#"Couldn't install dependencies for "{}": {}"#,
                self.package_json.name,
                error.to_string()
            ),
        }

        Ok(())
    }

    /// Runs the build script
    pub fn run_build(&self, config: &NpmWorkspaceConfig) -> Result<()> {
        if !self.package_json.scripts.contains_key(&config.build_script) {
            return Ok(());
        }

        action_step!(
            "Npm Build",
            "{} ({})",
            self.package_json.name,
            self.path.to_string_lossy()
        );

        let mut cmd = Command::new(&config.package_manager_path);

        let cmd = cmd
            .args(["run", &config.build_script])
            .current_dir(&self.path);

        match cmd.output() {
            Ok(output) if output.status.success() => {
                action_step!("Finished", "{}", self.package_json.name)
            }
            Ok(output) => error_step!(
                "Npm Build",
                r#"Couldn't build "{}": {}"#,
                self.package_json.name,
                // It's not a typo, it seems some package managers
                // use the stdout channel when an error occurs
                String::from_utf8(output.stdout).unwrap()
            ),
            Err(error) => error_step!(
                "Npm Build",
                r#"Couldn't build "{}": {}"#,
                self.package_json.name,
                error.to_string()
            ),
        }

        Ok(())
    }

    /// Runs the check script
    pub fn run_check(&self, config: &NpmWorkspaceConfig) -> Result<()> {
        if !self.package_json.scripts.contains_key(&config.check_script) {
            return Ok(());
        }

        action_step!(
            "Npm Check",
            "{} ({})",
            self.package_json.name,
            self.path.to_string_lossy()
        );

        let mut cmd = Command::new(&config.package_manager_path);

        let cmd = cmd
            .args(["run", &config.check_script])
            .current_dir(&self.path);

        match cmd.output() {
            Ok(output) if output.status.success() => {
                action_step!("Finished", "{}", self.package_json.name)
            }
            Ok(output) => error_step!(
                "Npm Check",
                r#"Check failed "{}": {}"#,
                self.package_json.name,
                // It's not a typo, it seems some package managers
                // use the stdout channel when an error occurs
                String::from_utf8(output.stdout).unwrap()
            ),
            Err(error) => error_step!(
                "Npm Check",
                r#"Check failed "{}": {}"#,
                self.package_json.name,
                error.to_string()
            ),
        }

        Ok(())
    }
}

/// Contain external data, such as the package manager binary path
/// or the build script name, that don't belong to the struct itself
#[derive(Debug)]
pub struct NpmWorkspaceConfig {
    /// The package manager binary path
    package_manager_path: PathBuf,
    /// The build script command (typically `build`)
    build_script: String,
    /// The check script command (typically `check`)
    check_script: String,
    /// The clean script command (typically `clean`)
    clean_script: String,
    /// The test script command (typically `test`)
    test_script: String,
}

/// References all the npm packages in a workspace
#[derive(Debug)]
pub struct NpmWorkspace {
    config: NpmWorkspaceConfig,
    /// The top level npm package
    root_package: NpmPackage,
    /// The npm packages, excluding the top level one
    packages: HashMap<String, NpmPackage>,
}

impl NpmWorkspace {
    /// Creates an empty workspace.
    pub fn empty(ctx: &Context) -> Result<Self> {
        let config = ctx.config();

        let package_manager_path = package_manager_path(&config.npm.package_manager)?;

        let root = ctx.workspace_root();

        let root_package = NpmPackage::from_path(root)?;

        let config = NpmWorkspaceConfig {
            package_manager_path,
            build_script: config.npm.build_script.clone(),
            check_script: config.npm.check_script.clone(),
            clean_script: config.npm.clean_script.clone(),
            test_script: config.npm.test_script.clone(),
        };

        Ok(Self {
            config,
            root_package,
            packages: HashMap::new(),
        })
    }

    /// Create a new workspace from the provided [`SelectedPackages`]
    pub fn from_selected_packages(
        ctx: &Context,
        selected_packages: &SelectedPackages,
    ) -> Result<Self> {
        let mut workspace = Self::empty(ctx)?;

        let packages = selected_packages.get_all_packages_metadata::<Metadata>(ctx)?;

        workspace.packages = packages
            .into_iter()
            .filter_map(|(path, metadata)| {
                let path = path.join(&metadata.npm.path);

                let npm_package = NpmPackage::from_path(path).ok()?;

                Some((metadata.npm.name, npm_package))
            })
            .collect::<HashMap<_, _>>();

        Ok(workspace)
    }

    /// Recursively searches for all the npm packages
    /// (folders that contain a `package.json` file)
    pub fn new(ctx: &Context) -> Result<Self> {
        let mut workspace = Self::empty(ctx)?;

        let config = ctx.config();

        let root = ctx.workspace_root();

        workspace.packages = WalkDir::new(&root)
            .into_iter()
            .filter_entry(|entry| {
                Self::entry_is_valid(entry, root.as_ref(), &config.npm.excluded_dirs)
            })
            .filter_map(|entry| {
                let entry = entry.ok().and_then(|entry| {
                    PackageJson::is_package_json(entry.file_name()).then(|| entry)
                })?;

                // The path not having a parent is very unlikely
                let path = entry.path().parent().unwrap();

                let npm_package = NpmPackage::from_path(path).ok()?;

                Some((npm_package.package_json.name.clone(), npm_package))
            })
            .collect();

        Ok(workspace)
    }

    fn entry_is_valid(entry: &DirEntry, root: &Path, excluded_dirs: &[String]) -> bool {
        let path = entry.path();
        let file_name = entry.file_name();

        // ignoring the package.json at the root
        if path == root.join(&PACKAGE_JSON)
            || excluded_dirs.iter().any(|dir| file_name == dir.as_str())
        {
            return false;
        }

        entry.path().is_dir() || PackageJson::is_package_json(entry.file_name())
    }

    pub fn run_install(&self) -> Result<()> {
        self.root_package.run_install(&self.config)
    }

    pub fn run_build(&self, package_name: &Option<String>) -> Result<()> {
        match package_name {
            None => self
                .packages
                .par_iter()
                .try_for_each(|(_, package)| package.run_build(&self.config)),
            Some(package_name) => match self.packages.get(package_name) {
                Some(package) => package.run_build(&self.config),
                None => Err(Error::new(format!(
                    "Couldn't find package {}",
                    package_name
                ))),
            },
        }
    }

    pub fn run_check(&self, package_name: &Option<String>) -> Result<()> {
        match package_name {
            None => self
                .packages
                .par_iter()
                .try_for_each(|(_, package)| package.run_check(&self.config)),
            Some(package_name) => match self.packages.get(package_name) {
                Some(package) => package.run_check(&self.config),
                None => Err(Error::new(format!(
                    "Couldn't find package {}",
                    package_name
                ))),
            },
        }
    }

    /// An npm workspace is empty is it doesn't contain any packages
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }
}

/// Returns the path to the package manager binary
pub fn package_manager_path(name: &str) -> Result<PathBuf> {
    which(name).map_err(|error| {
        Error::new(format!(r#"Package manager "{}" not found in PATH"#, name)).with_source(error)
    })
}
