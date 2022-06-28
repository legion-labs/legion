use std::fmt::Display;

use serde::Serialize;
use tabled::{Table, Tabled};

/// The serialization format.
#[derive(clap::ArgEnum, Clone, Debug)]
pub enum Format {
    /// JSON.
    Json,
    #[cfg(feature = "yaml")]
    /// YAML.
    Yaml,
    #[cfg(feature = "tabled")]
    /// Table.
    Table,
}

impl Default for Format {
    fn default() -> Self {
        Self::Table
    }
}

impl Format {
    pub fn format_unit<T: Display + Serialize>(&self, t: &T) {
        match self {
            Format::Json => {
                serde_json::to_writer_pretty(std::io::stdout(), t).unwrap();
                println!();
            }
            #[cfg(feature = "yaml")]
            Format::Yaml => {
                serde_yaml::to_writer(std::io::stdout(), t).unwrap();
                println!();
            }
            Format::Table => {
                println!("{}", t);
            }
        }
    }

    pub fn format_one<T: Tabled + Serialize>(&self, t: &T) {
        match self {
            Format::Json => {
                serde_json::to_writer_pretty(std::io::stdout(), t).unwrap();
                println!();
            }
            #[cfg(feature = "yaml")]
            Format::Yaml => {
                serde_yaml::to_writer(std::io::stdout(), t).unwrap();
                println!();
            }
            #[cfg(feature = "tabled")]
            Format::Table => {
                print!("{}", Table::new([t]).with(tabled::Style::psql()));
            }
        }
    }

    pub fn format_many<T: Tabled + Serialize>(&self, t: impl IntoIterator<Item = T>) {
        let t = t.into_iter().collect::<Vec<_>>();
        match self {
            Format::Json => {
                serde_json::to_writer_pretty(std::io::stdout(), &t).unwrap();
                println!();
            }
            #[cfg(feature = "yaml")]
            Format::Yaml => {
                serde_yaml::to_writer(std::io::stdout(), &t).unwrap();
                println!();
            }
            #[cfg(feature = "tabled")]
            Format::Table => {
                if t.is_empty() {
                    println!("The list is empty.");
                } else {
                    print!("{}", Table::new(t).with(tabled::Style::psql()));
                }
            }
        }
    }
}

pub(crate) fn optional<T: Display>(t: &Option<T>) -> String {
    match t {
        Some(t) => t.to_string(),
        None => "".to_string(),
    }
}
