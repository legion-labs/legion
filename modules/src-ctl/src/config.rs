use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub commands: BTreeMap<String, Vec<String>>,
    pub diff_commands: Vec<Vec<String>>,
    pub merge_commands: Vec<Vec<String>>,
}

impl Config {
    pub fn config_file_path() -> Result<PathBuf, String> {
        match dirs::config_dir() {
            Some(dir) => Ok(dir.join(".lsc")),
            None => Err(String::from("Error: no config directory")),
        }
    }

    pub fn read_config() -> Result<Config, String> {
        let path = Config::config_file_path()?;
        match read_text_file(&path) {
            Ok(contents) => {
                let parsed: serde_json::Result<Config> = serde_json::from_str(&contents);
                match parsed {
                    Ok(config) => Ok(config),
                    Err(e) => Err(format!("Error parsing config file: {}", e)),
                }
            }
            Err(e) => {
                return Err(format!(
                    "Error reading config file {}: {}",
                    path.display(),
                    e
                ));
            }
        }
    }

    fn find_command_impl(&self, command_spec_vec: &[Vec<String>], p: &Path) -> Option<Vec<String>> {
        let path_str = p.to_str().unwrap();
        for command_spec in command_spec_vec {
            match glob::Pattern::new(&command_spec[0]) {
                Ok(pattern) => {
                    if pattern.matches(&path_str) {
                        let command_name = &command_spec[1];
                        match self.commands.get(command_name) {
                            Some(command) => {
                                return Some(command.clone());
                            }
                            None => {
                                println!("unknown command named {}", command_name);
                                return None;
                            }
                        }
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

pub fn print_config_command() -> Result<(), String> {
    let path = Config::config_file_path()?;
    println!("config file path is {}", path.display());
    let config = Config::read_config()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&config).expect("formatting json")
    );
    Ok(())
}
