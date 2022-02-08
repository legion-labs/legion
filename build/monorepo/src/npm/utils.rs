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

const BUILD_SCRIPT: &str = "build";

const CHECK_SCRIPT: &str = "check";

const CLEAN_SCRIPT: &str = "clean";

const FORMAT_SCRIPT: &str = "fmt";

const FORMAT_CHECK_SCRIPT: &str = "fmt:check";

const TEST_SCRIPT: &str = "test";

const LINT_SCRIPT: &str = "lint";

const LINT_FIX_SCRIPT: &str = "lint:fix";

#[derive(Debug, Deserialize)]
struct NpmMetadata {
    name: String,
    path: Utf8PathBuf,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    npm: NpmMetadata,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug)]
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
                "Invalid {} file ({})",
                PACKAGE_JSON,
                path.to_string_lossy()
            ))
            .with_source(error)
        })?;

        Ok(Self::new(path, package_json))
    }

    pub fn run_install(&self, config: &NpmWorkspaceConfig) {
        action_step!(
            "Npm Install",
            "{} ({})",
            self.package_json.name,
            self.path.to_string_lossy()
        );

        let cmd_name = "Install";

        self.print_action_step(cmd_name);

        self.run_cmd(cmd_name, &config.package_manager_path, &[INSTALL_SCRIPT]);
    }

    /// Runs the build script
    pub fn run_build(&self, config: &NpmWorkspaceConfig) {
        if !self.package_json.scripts.contains_key(BUILD_SCRIPT) {
            return;
        }

        let cmd_name = "Build";

        self.print_action_step(cmd_name);

        self.run_cmd(
            cmd_name,
            &config.package_manager_path,
            &["run", BUILD_SCRIPT],
        );
    }

    /// Runs the check script
    pub fn run_check(&self, config: &NpmWorkspaceConfig) {
        if !self.package_json.scripts.contains_key(CHECK_SCRIPT) {
            return;
        }

        let cmd_name = "Check";

        self.print_action_step(cmd_name);

        self.run_cmd(
            cmd_name,
            &config.package_manager_path,
            &["run", CHECK_SCRIPT],
        );
    }

    /// Runs the clean script
    pub fn run_clean(&self, config: &NpmWorkspaceConfig) {
        if !self.package_json.scripts.contains_key(CLEAN_SCRIPT) {
            return;
        }

        let cmd_name = "Clean";

        self.print_action_step(cmd_name);

        self.run_cmd(
            cmd_name,
            &config.package_manager_path,
            &["run", CLEAN_SCRIPT],
        );
    }

    /// Runs the format script
    pub fn run_format(&self, config: &NpmWorkspaceConfig, check: bool) {
        if !self.package_json.scripts.contains_key(FORMAT_SCRIPT)
            || !self.package_json.scripts.contains_key(FORMAT_CHECK_SCRIPT)
        {
            return;
        }

        let cmd_name = "Format";

        self.print_action_step(cmd_name);

        let mut args = vec!["run"];

        if check {
            args.push(FORMAT_CHECK_SCRIPT);
        } else {
            args.push(FORMAT_SCRIPT);
        }

        self.run_cmd(cmd_name, &config.package_manager_path, &args);
    }

    /// Runs the lint script
    pub fn run_lint(&self, config: &NpmWorkspaceConfig, fix: bool) {
        if !self.package_json.scripts.contains_key(LINT_SCRIPT)
            || !self.package_json.scripts.contains_key(LINT_FIX_SCRIPT)
        {
            return;
        }

        let cmd_name = "Lint";

        self.print_action_step(cmd_name);

        let mut args = vec!["run"];

        if fix {
            args.push(LINT_FIX_SCRIPT);
        } else {
            args.push(LINT_SCRIPT);
        }

        self.run_cmd(cmd_name, &config.package_manager_path, &args);
    }

    /// Runs the test script
    pub fn run_test(&self, config: &NpmWorkspaceConfig) {
        if !self.package_json.scripts.contains_key(TEST_SCRIPT) {
            return;
        }

        let cmd_name = "Test";

        self.print_action_step(cmd_name);

        self.run_cmd(
            cmd_name,
            &config.package_manager_path,
            &["run", TEST_SCRIPT],
        );
    }

    fn print_action_step(&self, cmd_name: &str) {
        action_step!(
            &format!("Npm {}", cmd_name),
            "{} ({})",
            self.package_json.name,
            self.path.to_string_lossy()
        );
    }

    fn run_cmd(&self, cmd_name: &str, package_manager_path: &Path, args: &[&str]) {
        let mut cmd = Command::new(package_manager_path);

        let cmd = cmd.args(args).current_dir(&self.path);

        match cmd.output() {
            Ok(output) if output.status.success() => {
                action_step!("Finished", "{}", self.package_json.name)
            }
            Ok(output) => error_step!(
                &format!("Npm {}", cmd_name),
                "{}: {}\n{}",
                self.package_json.name,
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            ),
            Err(error) => error_step!(
                &format!("Npm {}", cmd_name),
                "{}: {}",
                self.package_json.name,
                error.to_string()
            ),
        }
    }
}

/// Contain external data, such as the package manager binary path
/// or the build script name, that don't belong to the struct itself
#[derive(Debug)]
pub struct NpmWorkspaceConfig {
    /// The package manager binary path
    package_manager_path: PathBuf,
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

    pub fn run_install(&self) {
        self.root_package.run_install(&self.config);
    }

    pub fn run_build(&self, package_name: &Option<String>) -> Result<()> {
        match package_name {
            None => {
                self.packages
                    .par_iter()
                    .for_each(|(_, package)| package.run_build(&self.config));

                Ok(())
            }
            Some(package_name) => match self.packages.get(package_name) {
                Some(package) => {
                    package.run_build(&self.config);

                    Ok(())
                }
                None => Err(Error::new(format!(
                    "Couldn't find package {}",
                    package_name
                ))),
            },
        }
    }

    pub fn run_check(&self, package_name: &Option<String>) -> Result<()> {
        match package_name {
            None => {
                self.packages
                    .par_iter()
                    .for_each(|(_, package)| package.run_check(&self.config));

                Ok(())
            }
            Some(package_name) => match self.packages.get(package_name) {
                Some(package) => {
                    package.run_check(&self.config);

                    Ok(())
                }
                None => Err(Error::new(format!(
                    "Couldn't find package {}",
                    package_name
                ))),
            },
        }
    }

    pub fn run_clean(&self, package_name: &Option<String>) -> Result<()> {
        match package_name {
            None => {
                self.root_package.run_clean(&self.config);

                self.packages
                    .par_iter()
                    .for_each(|(_, package)| package.run_clean(&self.config));

                Ok(())
            }
            Some(package_name) => match self.packages.get(package_name) {
                Some(package) => {
                    package.run_clean(&self.config);

                    Ok(())
                }
                None => Err(Error::new(format!(
                    "Couldn't find package {}",
                    package_name
                ))),
            },
        }
    }

    pub fn run_format(&self, package_name: &Option<String>, check: bool) -> Result<()> {
        match package_name {
            None => {
                self.packages
                    .par_iter()
                    .for_each(|(_, package)| package.run_format(&self.config, check));

                Ok(())
            }
            Some(package_name) => match self.packages.get(package_name) {
                Some(package) => {
                    package.run_format(&self.config, check);

                    Ok(())
                }
                None => Err(Error::new(format!(
                    "Couldn't find package {}",
                    package_name
                ))),
            },
        }
    }

    pub fn run_lint(&self, package_name: &Option<String>, fix: bool) -> Result<()> {
        match package_name {
            None => {
                self.packages
                    .par_iter()
                    .for_each(|(_, package)| package.run_lint(&self.config, fix));

                Ok(())
            }
            Some(package_name) => match self.packages.get(package_name) {
                Some(package) => {
                    package.run_lint(&self.config, fix);

                    Ok(())
                }
                None => Err(Error::new(format!(
                    "Couldn't find package {}",
                    package_name
                ))),
            },
        }
    }

    pub fn run_test(&self, package_name: &Option<String>) -> Result<()> {
        match package_name {
            None => {
                self.packages
                    .par_iter()
                    .for_each(|(_, package)| package.run_test(&self.config));

                Ok(())
            }
            Some(package_name) => match self.packages.get(package_name) {
                Some(package) => {
                    package.run_test(&self.config);

                    Ok(())
                }
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
        Error::new(format!("Package manager {} not found in PATH", name)).with_source(error)
    })
}
