use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

use which::which;

use crate::{
    error::{Error, Result},
    types::ElectronRuntimeConfiguration,
};

const ENV_KEY: &str = "ELECTRON_RUNTIME_CONFIGURATION";

#[derive(Debug)]
pub struct ElectronPackageConfigTypeScript {
    root: PathBuf,
    tsconfig: PathBuf,
}

#[derive(Debug)]
pub enum ElectronPackageConfigType {
    JavaScript,
    TypeScript(ElectronPackageConfigTypeScript),
}

impl ElectronPackageConfigType {
    pub fn get_typescript(&self) -> Option<&ElectronPackageConfigTypeScript> {
        match self {
            Self::TypeScript(typescript) => Some(typescript),
            Self::JavaScript => None,
        }
    }

    pub fn is_typescript(&self) -> bool {
        self.get_typescript().is_some()
    }
}

#[derive(Debug)]
pub struct ElectronPackageConfig {
    /// Root of the node package itself
    root: PathBuf,
    /// Absolute path to the main script, always a valid Electron JavaScript file
    main: PathBuf,
    typ: ElectronPackageConfigType,
}

impl ElectronPackageConfig {
    pub fn new(
        root: impl AsRef<Path>,
        main: impl AsRef<Path>,
        tsconfig: impl AsRef<Path>,
        force_typescript: bool,
    ) -> Result<Self> {
        let root = dunce::canonicalize(root)?;

        if !root.is_dir() {
            return Err(Error::PathIsNotADir(root));
        }

        let original_main = main.as_ref();

        let main_is_typescript = original_main.extension().and_then(OsStr::to_str) == Some("ts");

        let is_typescript = force_typescript || main_is_typescript;

        let main = if is_typescript {
            let main = if main_is_typescript {
                original_main.with_extension("js")
            } else {
                original_main.into()
            };

            let file_name = main.clone();

            let file_name = file_name.file_name().ok_or(Error::PathIsNotAFile(main))?;

            Path::new("tmp").join(file_name)
        } else {
            if !original_main.is_file() {
                return Err(Error::PathIsNotAFile(original_main.into()));
            }

            original_main.into()
        };

        let main = root.join(main);

        let typ = if is_typescript {
            ElectronPackageConfigType::TypeScript(ElectronPackageConfigTypeScript {
                root: root
                    .join(original_main)
                    .parent()
                    .ok_or_else(|| Error::PathIsNotAFile(original_main.into()))?
                    .into(),
                tsconfig: root.join(tsconfig),
            })
        } else {
            ElectronPackageConfigType::JavaScript
        };

        Ok(Self { root, main, typ })
    }

    /// Returns the "local" package config, that is, the one located in this exact crate
    pub fn local() -> Result<Self> {
        let absolute_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("electron");

        Self::new(absolute_root, "src/main.ts", "tsconfig.json", true)
    }

    pub fn is_typescript(&self) -> bool {
        self.typ.is_typescript()
    }
}

pub struct NpxCommand<'a> {
    command: Command,
    electron_package_config: &'a ElectronPackageConfig,
}

impl<'a> NpxCommand<'a> {
    pub fn new<'b: 'a>(electron_package_config: &'b ElectronPackageConfig) -> Result<Self> {
        let binary_path = which("npx").map_err(Error::from)?;

        let mut command = Command::new(binary_path);

        command.current_dir(&electron_package_config.root);

        Ok(Self {
            command,
            electron_package_config,
        })
    }

    /// Runs the Electron binary with the provided runtime configuration
    pub fn run_electron(mut self, configuration: &ElectronRuntimeConfiguration) -> Result<()> {
        self.command
            .args([
                "electron",
                &self.electron_package_config.main.to_string_lossy(),
            ])
            .env(ENV_KEY, configuration.serialize()?);

        let status = self.command.status().map_err(Error::from)?;

        if status.success() {
            Ok(())
        } else {
            Err(Error::ElectronCommandFailed(status))
        }
    }

    /// Runs the Electron binary with the provided runtime configuration
    pub fn run_tsc(mut self) -> Result<()> {
        let typ = match self.electron_package_config.typ {
            ElectronPackageConfigType::TypeScript(ref typ) => typ,
            ElectronPackageConfigType::JavaScript => return Ok(()),
        };

        self.command.args([
            "tsc",
            "--rootDir",
            &typ.root.to_string_lossy(),
            "--outDir",
            "tmp",
            "--noEmit",
            "false",
            "--project",
            &typ.tsconfig.to_string_lossy(),
        ]);

        let status = self.command.status().map_err(Error::from)?;

        if status.success() {
            Ok(())
        } else {
            Err(Error::ElectronCommandFailed(status))
        }
    }
}
