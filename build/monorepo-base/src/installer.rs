// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error_step;
use crate::{action_step, config::CargoInstallation, skip_step};
use std::collections::HashMap;
use std::process::{Command, Stdio};

pub struct Installer {
    cargo_installations: HashMap<String, CargoInstallation>,
}

impl Installer {
    pub fn new(cargo_installations: HashMap<String, CargoInstallation>) -> Self {
        Self {
            cargo_installations,
        }
    }

    pub fn install_via_cargo_if_needed(&self, name: &str) -> bool {
        match &self.cargo_installations.get(name) {
            Some(cargo_installation) => install_cargo_component_if_needed(name, cargo_installation),
            None => {
                skip_step!("Installer", "No installation for {}", name);
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
                skip_step!("Installer", "No installation for {}", name);
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

    pub fn install_all(&self) -> bool {
        let iter = self
            .cargo_installations
            .iter()
            .collect::<Vec<(&String, &CargoInstallation)>>();
        install_all_cargo_components(iter.as_slice())
    }
}

fn install_cargo_component_if_needed(name: &str, installation: &CargoInstallation) -> bool {
    if !check_installed_cargo_component(name, &installation.version) {
        action_step!("Installer", "Installing {} {}", name, installation.version);
        //prevent recursive install attempts of sccache.
        let mut cmd = Command::new("cargo");
        cmd.arg("install");
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
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        let result = cmd.output();
        if result.is_err() || !result.unwrap().status.success() {
            error_step!(
                "Installer",
                "Could not install {} {}, check x.toml to ensure tool exists and is not yanked, or provide a git-rev if your x.toml specifies a git-url.",
                name, installation.version
            );
            false
        } else {
            action_step!("Installer", "Installed {} {}", name, installation.version);
            true
        }
    } else {
        skip_step!("Installer", "{} already installed", name);
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
    action_step!(
        "Installer",
        "{} of version {} is{} installed",
        name,
        version,
        if !found { " not" } else { "" }
    );
    found
}

pub fn install_all_cargo_components(tools: &[(&String, &CargoInstallation)]) -> bool {
    let mut success: bool = true;
    for (name, installation) in tools {
        success &= install_cargo_component_if_needed(name, installation);
    }
    success
}

pub fn check_all_cargo_components(tools: &[(&String, &String)]) -> bool {
    let mut success: bool = true;
    for (key, value) in tools {
        success &= check_installed_cargo_component(key, value);
    }
    success
}
