// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use camino::{Utf8Path, Utf8PathBuf};
use guppy::graph::PackageGraph;
use lgn_tracing::span_fn;
use lgn_tracing::span_scope;
use monorepo_base::config::MONOREPO_DEPTH;
use monorepo_base::installer::Installer;
use once_cell::sync::OnceCell;

use crate::cargo::current_target_cfg;
use crate::config::MonorepoConfig;
use crate::git::GitCli;
use crate::Error;
use crate::Result;

pub struct Context {
    workspace_root: &'static Utf8Path,
    current_dir: Utf8PathBuf,
    current_rel_dir: Utf8PathBuf,
    config: MonorepoConfig,
    installer: Installer,
    package_graph: OnceCell<PackageGraph>,
    git_cli: OnceCell<GitCli>,
    target_config: OnceCell<String>,
}

impl Context {
    #[span_fn]
    pub fn new() -> Result<Self> {
        let workspace_root = Utf8Path::new(&env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(MONOREPO_DEPTH)
            .unwrap();

        let current_dir: Utf8PathBuf = std::env::current_dir()
            .map_err(|err| Error::new("error while fetching current dir").with_source(err))?
            .try_into()
            .map_err(|err| Error::new("current dir is not valid UTF-8").with_source(err))?;

        let current_rel_dir = match current_dir.strip_prefix(workspace_root) {
            Ok(rel_dir) => rel_dir.to_path_buf(),
            Err(_) => {
                return Err(Error::new(format!(
                    "Current directory {} not in workspace {}",
                    current_dir, workspace_root,
                )))
            }
        };

        let config = MonorepoConfig::new(workspace_root)?;
        let installer = Installer::new(config.cargo.installs.clone());

        Ok(Self {
            workspace_root,
            current_dir,
            current_rel_dir,
            config,
            installer,
            package_graph: OnceCell::new(),
            git_cli: OnceCell::new(),
            target_config: OnceCell::new(),
        })
    }
    pub fn config(&self) -> &MonorepoConfig {
        &self.config
    }

    /// Returns a reference to Installer, configured to install versions from config.
    pub fn installer(&self) -> &Installer {
        &self.installer
    }

    pub fn package_graph(&self) -> Result<&PackageGraph> {
        self.package_graph.get_or_try_init(|| {
            span_scope!("Context::package_graph::init");
            let mut cmd = guppy::MetadataCommand::new();
            cmd.current_dir(self.workspace_root);
            guppy::graph::PackageGraph::from_command(&mut cmd)
                .map_err(|err| Error::new("failed to build package graph").with_source(err))
        })
    }

    pub fn git_cli(&self) -> Result<&GitCli> {
        self.git_cli.get_or_try_init(|| {
            span_scope!("Context::git_cli::init");
            GitCli::new(self.workspace_root())
        })
    }

    pub fn workspace_root(&self) -> &'static Utf8Path {
        self.workspace_root
    }

    pub fn target_config(&self) -> Result<&String> {
        self.target_config.get_or_try_init(|| {
            span_scope!("Context::target_config::init");
            current_target_cfg()
        })
    }

    /// Returns the current working directory for this process.
    #[allow(dead_code)]
    pub fn current_dir(&self) -> &Utf8Path {
        &self.current_dir
    }

    /// Returns the current working directory for this process, relative to the project root.
    pub fn current_rel_dir(&self) -> &Utf8Path {
        &self.current_rel_dir
    }

    /// Returns true if x has been run from the project root.
    pub fn current_dir_is_root(&self) -> bool {
        self.current_rel_dir == ""
    }

    /// For a given list of workspace packages, returns a tuple of (known, unknown) packages.
    ///
    /// Initializes the package graph if it isn't already done so, and returns an error if the
    #[span_fn]
    pub fn partition_workspace_names<'a, B>(
        &self,
        names: impl IntoIterator<Item = &'a str>,
    ) -> Result<(B, B)>
    where
        B: Default + Extend<&'a str>,
    {
        let workspace = self.package_graph()?.workspace();
        let (known, unknown) = names
            .into_iter()
            .partition(|name| workspace.contains_name(name));
        Ok((known, unknown))
    }
}
