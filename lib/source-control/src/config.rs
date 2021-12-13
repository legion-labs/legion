use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::{read_text_file, trace_scope};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub commands: BTreeMap<String, Vec<String>>,
    pub diff_commands: Vec<Vec<String>>,
    pub merge_commands: Vec<Vec<String>>,
}

impl Config {
    pub fn config_file_path() -> Result<PathBuf> {
        if let Some(dir) = dirs::config_dir() {
            Ok(dir.join(".lsc"))
        } else {
            anyhow::bail!("no config directory");
        }
    }

    pub fn read_config() -> Result<Self> {
        let path = Self::config_file_path()?;
        let contents =
            read_text_file(&path).context(format!("read config file {}", path.display()))?;

        serde_json::from_str(&contents).context("failed to parse config file")
    }

    fn find_command_impl(&self, command_spec_vec: &[Vec<String>], p: &Path) -> Option<Vec<String>> {
        let path_str = p.to_str().unwrap();
        for command_spec in command_spec_vec {
            match glob::Pattern::new(&command_spec[0]) {
                Ok(pattern) => {
                    if pattern.matches(path_str) {
                        let command_name = &command_spec[1];
                        if let Some(command) = self.commands.get(command_name) {
                            return Some(command.clone());
                        }
                        println!("unknown command named {}", command_name);
                        return None;
                    }
                }
                Err(e) => {
                    println!("Error parsing pattern {}: {}", &command_spec[0], e);
                }
            }
        }
        None
    }

    pub fn find_diff_command(&self, p: &Path) -> Option<Vec<String>> {
        self.find_command_impl(&self.diff_commands, p)
    }

    pub fn find_merge_command(&self, p: &Path) -> Option<Vec<String>> {
        self.find_command_impl(&self.merge_commands, p)
    }
}

pub fn print_config_command() -> Result<()> {
    trace_scope!();
    let path = Config::config_file_path()?;
    println!("config file path is {}", path.display());
    let config = Config::read_config()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&config).expect("formatting json")
    );
    Ok(())
}
