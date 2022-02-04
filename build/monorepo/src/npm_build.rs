use std::{
    borrow::Cow,
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use camino::Utf8Path;
use lgn_tracing::span_fn;
use monorepo_base::{action_step, error_step, skip_step};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::Deserialize;
use walkdir::{DirEntry, WalkDir};
use which::which;

use crate::{context::Context, error::Error, Result};

const PACKAGE_JSON: &str = "package.json";

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(long, short)]
    pub(crate) package_name: Option<String>,
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
struct NpmPackage<'a> {
    package_manager_path: &'a Path,
    path: PathBuf,
    package_json: PackageJson,
}

impl<'a> NpmPackage<'a> {
    fn new<'b, P: Into<Cow<'b, Path>>>(
        package_manager_path: &'a Path,
        path: P,
        package_json: PackageJson,
    ) -> Self {
        Self {
            package_manager_path,
            path: path.into().into_owned(),
            package_json,
        }
    }

    /// Initialize an [`NpmPackage`] from a path
    fn from_path(package_manager_path: &'a Path, path: &Path) -> Result<Self> {
        let path = path.to_path_buf();

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

        Ok(Self::new(package_manager_path, path, package_json))
    }

    fn name_is(&self, name: &str) -> bool {
        self.package_json.name == name
    }

    /// Runs a build script. Returns `Ok(true)` if the script could be found and ran
    /// `Ok(false)` if the script was not found in the package.json file and
    /// an `Err(Error)` otherwise
    fn run_build(&self, build_script: &str) -> Result<bool> {
        if !self.package_json.scripts.contains_key(build_script) {
            skip_step!(
                "Npm Build",
                r#"{} ({})"#,
                self.package_json.name,
                self.path.to_string_lossy()
            );

            return Ok(false);
        }

        action_step!(
            "Npm Build",
            "{} ({})",
            self.package_json.name,
            self.path.to_string_lossy()
        );

        let mut cmd = Command::new(self.package_manager_path);

        let cmd = cmd.arg("run").arg(build_script).current_dir(&self.path);

        match cmd.output() {
            Ok(Output { status, .. }) if status.success() => {
                action_step!("Finished", "{}", self.package_json.name)
            }
            Ok(Output { stdout, .. }) => error_step!(
                "Npm Build",
                r#"Couldn't build "{}": {}"#,
                self.package_json.name,
                // It's not a typo, it seems some package managers
                // use the stdout channel when an error occurs
                String::from_utf8(stdout).unwrap()
            ),
            Err(error) => error_step!(
                "Npm Build",
                r#"Couldn't build "{}": {}"#,
                self.package_json.name,
                error.to_string()
            ),
        }

        Ok(true)
    }
}

/// Simple 0-cost wrapper around [`DirEntry`]
struct Entry<'a>(&'a DirEntry);

impl<'a> Entry<'a> {
    fn is_valid(&self, root: &Utf8Path, excluded_dirs: &[String]) -> bool {
        let path = self.0.path();
        let file_name = self.0.file_name();

        // ignoring the package.json at the root
        if path == root.join(&PACKAGE_JSON)
            || excluded_dirs.iter().any(|dir| file_name == dir.as_str())
        {
            return false;
        }

        self.0.path().is_dir() || PackageJson::is_package_json(self.0.file_name())
    }
}

#[span_fn]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let config = ctx.config();

    let package_manager_path = which(&config.npm.package_manager).map_err(|error| {
        Error::new(format!(
            r#"Package manager "{}" not found in PATH"#,
            config.npm.package_manager
        ))
        .with_source(error)
    })?;

    let root = ctx.workspace_root();

    // Get all valid npm module paths (i.e. dir that contain a `package.json` file)
    let package_entry = WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| Entry(entry).is_valid(root, &config.npm.excluded_dirs))
        .filter_map(|entry| {
            entry
                .ok()
                .and_then(|entry| PackageJson::is_package_json(entry.file_name()).then(|| entry))
        })
        .collect::<Vec<_>>();

    match args.package_name {
        Some(ref package_name) => {
            let matching_dir_entry = package_entry.into_par_iter().find_first(|entry| {
                // The path not having a parent is highly unlikely
                let path = entry.path().parent().unwrap();

                let npm_package = NpmPackage::from_path(&package_manager_path, path)
                    .ok()
                    .and_then(|npm_package| npm_package.name_is(package_name).then(|| npm_package));

                match npm_package {
                    None => false,
                    Some(npm_package) => {
                        match npm_package.run_build(&config.npm.build_script) {
                            Ok(_) => (),
                            Err(error) => error_step!(
                                "Npm Build",
                                r#"An error occurred while running the build script "{}""#,
                                error.to_string()
                            ),
                        };

                        true
                    }
                }
            });

            if matching_dir_entry.is_none() {
                error_step!("Npm Build", "Couldn't find package {}", package_name);
            }
        }

        None => {
            package_entry.into_par_iter().for_each(|entry| {
                // The path not having a parent is highly unlikely
                let path = entry.path().parent().unwrap();

                match NpmPackage::from_path(&package_manager_path, path) {
                    Ok(npm_package) => {
                        match npm_package.run_build(&config.npm.build_script) {
                            Ok(_) => (),
                            Err(error) => error_step!(
                                "Npm Build",
                                r#"An error occurred while running the build script "{}""#,
                                error.to_string()
                            ),
                        };
                    }
                    Err(_error) => error_step!(
                        "Npm Build",
                        r#"Couldn't initialize npm package at "{}""#,
                        path.to_string_lossy()
                    ),
                };
            });
        }
    }

    Ok(())
}
