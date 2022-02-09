use std::{
    borrow::Cow,
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    process::Command,
};

use camino::{Utf8Path, Utf8PathBuf};
use monorepo_base::{action_step, error_step, skip_step};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use walkdir::{DirEntry, WalkDir};
use which::which;

use crate::{
    cargo::SelectedPackages, config::MonorepoConfig, context::Context, error::Error, Result,
};

const DIST_FOLDER: &str = "dist";

const SRC_FOLDER: &str = "src";

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
    fn is_package_json<'a, F: PartialEq<&'a str>>(file_name: &F) -> bool {
        *file_name == PACKAGE_JSON
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

    pub fn install<P: AsRef<Path>>(&self, package_manager_path: P) {
        action_step!(
            "Npm Install",
            "{} ({})",
            self.package_json.name,
            self.path.to_string_lossy()
        );

        let cmd_name = "Install";

        self.print_action_step(cmd_name);

        self.run_cmd(cmd_name, package_manager_path.as_ref(), &[INSTALL_SCRIPT]);
    }

    /// Runs the build script
    pub fn build<P: AsRef<Path>>(&self, package_manager_path: P) {
        if !self.package_json.scripts.contains_key(BUILD_SCRIPT) {
            return;
        }

        let cmd_name = "Build";

        if !self.should_build() {
            self.print_skip_step(cmd_name);

            return;
        }

        self.print_action_step(cmd_name);

        self.run_cmd(
            cmd_name,
            package_manager_path.as_ref(),
            &["run", BUILD_SCRIPT],
        );
    }

    /// Runs the check script
    pub fn check<P: AsRef<Path>>(&self, package_manager_path: P) {
        if !self.package_json.scripts.contains_key(CHECK_SCRIPT) {
            return;
        }

        let cmd_name = "Check";

        self.print_action_step(cmd_name);

        self.run_cmd(
            cmd_name,
            package_manager_path.as_ref(),
            &["run", CHECK_SCRIPT],
        );
    }

    /// Runs the clean script
    pub fn clean<P: AsRef<Path>>(&self, package_manager_path: P) {
        if !self.package_json.scripts.contains_key(CLEAN_SCRIPT) {
            return;
        }

        let cmd_name = "Clean";

        self.print_action_step(cmd_name);

        self.run_cmd(
            cmd_name,
            package_manager_path.as_ref(),
            &["run", CLEAN_SCRIPT],
        );
    }

    /// Runs the format script
    pub fn format<P: AsRef<Path>>(&self, package_manager_path: P, check: bool) {
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

        self.run_cmd(cmd_name, package_manager_path.as_ref(), &args);
    }

    /// Runs the lint script
    pub fn lint<P: AsRef<Path>>(&self, package_manager_path: P, fix: bool) {
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

        self.run_cmd(cmd_name, package_manager_path.as_ref(), &args);
    }

    /// Runs the test script
    pub fn test<P: AsRef<Path>>(&self, package_manager_path: P) {
        if !self.package_json.scripts.contains_key(TEST_SCRIPT) {
            return;
        }

        let cmd_name = "Test";

        self.print_action_step(cmd_name);

        self.run_cmd(
            cmd_name,
            package_manager_path.as_ref(),
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

    fn print_skip_step(&self, cmd_name: &str) {
        skip_step!(
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
                action_step!("Finished", "{}", self.package_json.name);
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

    /// Checks whether or not a build is needed.
    /// In order to check this, the function simply compares
    /// the last modification date of the `dist` directory and
    /// the package src directory content. If an error occurs
    /// in the process, `true` is returned so the build can occur.
    fn should_build(&self) -> bool {
        let dist_modified =
            match fs::metadata(&self.path.join(DIST_FOLDER)).and_then(|m| m.modified()) {
                Ok(mtime) => mtime,
                Err(_error) => {
                    return true;
                }
            };

        let package_modified = WalkDir::new(&self.path.join(SRC_FOLDER))
            .into_iter()
            .filter_map(|file| {
                let file = file.ok()?;

                let metadata = fs::metadata(file.path()).ok()?;

                metadata.modified().ok()
            })
            .max();

        let package_modified = match package_modified {
            None => return true,
            Some(package_modified) => package_modified,
        };

        dist_modified < package_modified
    }
}

/// References all the npm packages in a workspace
pub struct NpmWorkspace<'a> {
    /// Borrow the whole context to access
    /// some data like the root path, the npm config, etc...
    ctx: &'a Context,
    /// Owned path to the package manager binary
    package_manager_path: PathBuf,
    /// The top level npm package
    root_package: NpmPackage,
    /// The npm packages, excluding the top level one
    packages: HashMap<String, NpmPackage>,
}

impl<'a> NpmWorkspace<'a> {
    /// Creates an empty workspace.
    pub fn new(ctx: &'a Context) -> Result<Self> {
        let config = ctx.config();

        let package_manager_path = which(&config.npm.package_manager).map_err(|error| {
            Error::new(format!(
                "Package manager {} not found in PATH",
                &config.npm.package_manager
            ))
            .with_source(error)
        })?;

        let root = ctx.workspace_root();

        let root_package = NpmPackage::from_path(root)?;

        Ok(Self {
            ctx,
            package_manager_path,
            root_package,
            packages: HashMap::new(),
        })
    }

    /// Populates the workspace using the provided [`SelectedPackages`]
    pub fn load_selected_packages(
        &mut self,
        selected_packages: &SelectedPackages<'_>,
    ) -> Result<()> {
        let packages = selected_packages.get_all_packages_metadata::<Metadata>(self.ctx)?;

        self.packages = packages
            .into_iter()
            .filter_map(|(path, metadata)| {
                let path = path.join(&metadata.npm.path);

                let npm_package = NpmPackage::from_path(path).ok()?;

                Some((metadata.npm.name, npm_package))
            })
            .collect::<HashMap<_, _>>();

        Ok(())
    }

    /// Recursively searches for all the npm packages to populate the workspace
    /// (folders that contain a `package.json` file)
    pub fn load_all(&mut self) {
        let config = self.config();
        let root = self.root();

        self.packages = config
            .npm
            .include
            .par_iter()
            .flat_map(|include| {
                WalkDir::new(root.join(include))
                    .into_iter()
                    .filter_entry(|entry| self.entry_is_valid(entry))
                    .filter_map(|entry| {
                        println!("entry {:?}", entry);

                        let entry = entry.ok().and_then(|entry| {
                            PackageJson::is_package_json(&entry.file_name()).then(|| entry)
                        })?;

                        // The path not having a parent is very unlikely
                        let path = entry.path().parent().unwrap();

                        let npm_package = NpmPackage::from_path(path).ok()?;

                        Some((npm_package.package_json.name.clone(), npm_package))
                    })
                    .collect::<HashMap<_, _>>()
            })
            .collect();
    }

    fn entry_is_valid(&self, entry: &DirEntry) -> bool {
        let root = self.root();
        let path = entry.path();
        let file_name = entry.file_name();

        if path == root.join(&PACKAGE_JSON) {
            return false;
        }

        entry.path().is_dir() || PackageJson::is_package_json(&file_name)
    }

    pub fn install(&self) {
        self.root_package.install(&self.package_manager_path);
    }

    pub fn build(&self, package_name: &Option<String>) -> Result<()> {
        match package_name {
            None => {
                self.packages
                    .par_iter()
                    .for_each(|(_, package)| package.build(&self.package_manager_path));

                Ok(())
            }
            Some(package_name) => match self.packages.get(package_name) {
                Some(package) => {
                    package.build(&self.package_manager_path);

                    Ok(())
                }
                None => Err(Error::new(format!(
                    "Couldn't find package {}",
                    package_name
                ))),
            },
        }
    }

    pub fn check(&self, package_name: &Option<String>) -> Result<()> {
        match package_name {
            None => {
                self.packages
                    .par_iter()
                    .for_each(|(_, package)| package.check(&self.package_manager_path));

                Ok(())
            }
            Some(package_name) => match self.packages.get(package_name) {
                Some(package) => {
                    package.check(&self.package_manager_path);

                    Ok(())
                }
                None => Err(Error::new(format!(
                    "Couldn't find package {}",
                    package_name
                ))),
            },
        }
    }

    pub fn clean(&self, package_name: &Option<String>) -> Result<()> {
        match package_name {
            None => {
                self.root_package.clean(&self.package_manager_path);

                self.packages
                    .par_iter()
                    .for_each(|(_, package)| package.clean(&self.package_manager_path));

                Ok(())
            }
            Some(package_name) => match self.packages.get(package_name) {
                Some(package) => {
                    package.clean(&self.package_manager_path);

                    Ok(())
                }
                None => Err(Error::new(format!(
                    "Couldn't find package {}",
                    package_name
                ))),
            },
        }
    }

    pub fn format(&self, package_name: &Option<String>, check: bool) -> Result<()> {
        match package_name {
            None => {
                self.packages
                    .par_iter()
                    .for_each(|(_, package)| package.format(&self.package_manager_path, check));

                Ok(())
            }
            Some(package_name) => match self.packages.get(package_name) {
                Some(package) => {
                    package.format(&self.package_manager_path, check);

                    Ok(())
                }
                None => Err(Error::new(format!(
                    "Couldn't find package {}",
                    package_name
                ))),
            },
        }
    }

    pub fn lint(&self, package_name: &Option<String>, fix: bool) -> Result<()> {
        match package_name {
            None => {
                self.packages
                    .par_iter()
                    .for_each(|(_, package)| package.lint(&self.package_manager_path, fix));

                Ok(())
            }
            Some(package_name) => match self.packages.get(package_name) {
                Some(package) => {
                    package.lint(&self.package_manager_path, fix);

                    Ok(())
                }
                None => Err(Error::new(format!(
                    "Couldn't find package {}",
                    package_name
                ))),
            },
        }
    }

    pub fn test(&self, package_name: &Option<String>) -> Result<()> {
        match package_name {
            None => {
                self.packages
                    .par_iter()
                    .for_each(|(_, package)| package.test(&self.package_manager_path));

                Ok(())
            }
            Some(package_name) => match self.packages.get(package_name) {
                Some(package) => {
                    package.test(&self.package_manager_path);

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

    fn root(&self) -> &Utf8Path {
        self.ctx.workspace_root()
    }

    fn config(&self) -> &MonorepoConfig {
        self.ctx.config()
    }
}
