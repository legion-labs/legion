// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{cargo::Cargo, config::CargoInstallation, context::Context, ignore_step};
use lgn_telemetry::{error, info};
use std::{collections::HashMap, process::Command};

pub struct Installer {
    cargo_installations: HashMap<String, CargoInstallation>,
}

impl Installer {
    pub fn new(cargo_installations: HashMap<String, CargoInstallation>) -> Self {
        Self {
            cargo_installations,
        }
    }

    pub fn install_via_cargo_if_needed(&self, ctx: &Context, name: &str) -> bool {
        match &self.cargo_installations.get(name) {
            Some(cargo_installation) => {
                install_cargo_component_if_needed(ctx, name, cargo_installation)
            }
            None => {
                ignore_step!("Installer", "No installation for {}", name);
                false
            }
        }
    }

    #[allow(dead_code)]
    fn check_cargo_component(&self, name: &str) -> bool {
        match &self.cargo_installations.get(name) {
            Some(cargo_installation) => {
                check_installed_cargo_component(name, &cargo_installation.version)
            }
            None => {
                ignore_step!("Installer", "No installation for {}", name);
                false
            }
        }
    }

    pub fn check_all(&self) -> bool {
        let iter = self
            .cargo_installations
            .iter()
            .map(|(name, installation)| (name, &installation.version))
            .collect::<Vec<(&String, &String)>>();
        check_all_cargo_components(iter.as_slice())
    }

    pub fn install_all(&self, ctx: &Context) -> bool {
        let iter = self
            .cargo_installations
            .iter()
            .collect::<Vec<(&String, &CargoInstallation)>>();
        install_all_cargo_components(ctx, iter.as_slice())
    }
}

fn install_cargo_component_if_needed(
    ctx: &Context,
    name: &str,
    installation: &CargoInstallation,
) -> bool {
    if !check_installed_cargo_component(name, &installation.version) {
        info!("Installing {} {}", name, installation.version);
        //prevent recursive install attempts of sccache.
        let mut cmd = Cargo::new(ctx, "install", true);
        cmd.arg("--force");
        if let Some(features) = &installation.features {
            if !features.is_empty() {
                cmd.arg("--features");
                cmd.args(features);
            }
        }
        if let Some(git_url) = &installation.git {
            cmd.arg("--git");
            cmd.arg(git_url);
            if let Some(git_rev) = &installation.git_rev {
                cmd.arg("--rev");
                cmd.arg(git_rev);
            }
        } else {
            cmd.arg("--version").arg(&installation.version);
        }
        cmd.arg("--locked");
        cmd.arg(name);

        let result = cmd.run();
        if result.is_err() {
            error!(
                "Could not install {} {}, check x.toml to ensure tool exists and is not yanked, or provide a git-rev if your x.toml specifies a git-url.",
                name, installation.version
            );
        }
        result.is_ok()
    } else {
        true
    }
}

//TODO check installed features for sccache?
fn check_installed_cargo_component(name: &str, version: &str) -> bool {
    let result = Command::new(name).arg("--version").output();
    let found = match result {
        Ok(output) => {
            let output = String::from_utf8_lossy(output.stdout.as_slice());
            format!("{} {}", name, version).eq(output.trim())
                || format!("{} v{}", name, version).eq(output.trim())
        }
        _ => false,
    };
    info!(
        "{} of version {} is{} installed",
        name,
        version,
        if !found { " not" } else { "" }
    );
    found
}

fn install_all_cargo_components(ctx: &Context, tools: &[(&String, &CargoInstallation)]) -> bool {
    let mut success: bool = true;
    for (name, installation) in tools {
        success &= install_cargo_component_if_needed(ctx, name, installation);
    }
    success
}

fn check_all_cargo_components(tools: &[(&String, &String)]) -> bool {
    let mut success: bool = true;
    for (key, value) in tools {
        success &= check_installed_cargo_component(key, value);
    }
    success
}
