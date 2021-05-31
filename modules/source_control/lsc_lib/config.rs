use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub commands: BTreeMap<String, String>,
    pub diff_commands: Vec<Vec<String>>,
}

pub fn config_file_path() -> Result<std::path::PathBuf, String> {
    match dirs::config_dir() {
        Some(dir) => Ok(dir.join(".lsc")),
        None => Err(String::from("Error: no config directory")),
    }
}

pub fn print_config_command() -> Result<(), String> {
    match dirs::config_dir() {
        Some(dir) => {
            let path = dir.join(".lsc");
            println!("config file path is {}", path.display());
            let contents = read_text_file(&path)?;
            let parsed: serde_json::Result<Config> = serde_json::from_str(&contents);
            match parsed {
                Ok(config) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&config).expect("formatting json")
                    );
                }
                Err(e) => {
                    return Err(format!("Error parsing config file: {}", e));
                }
            }
        }
        None => {
            return Err(String::from("no config directory"));
        }
    }
    Ok(())
}
