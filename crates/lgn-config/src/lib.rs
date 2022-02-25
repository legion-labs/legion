//! config create exposes the legion config file to applications

// crate-specific lint exceptions:
#![allow(clippy::missing_errors_doc)]

use std::env;
use std::mem::discriminant;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use lgn_tracing::warn;
use toml::value::{Table, Value};

const DEFAULT_CONFIG_FILENAME: &str = "legion.toml";
const LOCAL_CONFIG_FILENAME: &str = "legion_local.toml";

pub struct Config {
    config_path: PathBuf,
    entries: Table,
}

impl Config {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        // Search for the CONFIG file from the current exec direction,
        // walking up to the parents directory
        let mut config_dir = env::current_exe().unwrap();
        config_dir.pop();

        let mut global_table: Option<Table> = None;

        loop {
            let default_config = config_dir.join(DEFAULT_CONFIG_FILENAME);
            if default_config.is_file() {
                if let Some(mut default_table) = Self::load_toml_table(default_config.as_path()) {
                    let local_config = config_dir.join(LOCAL_CONFIG_FILENAME);
                    if local_config.is_file() {
                        // apply local override
                        if let Some(overrides) = Self::load_toml_table(local_config.as_path()) {
                            Self::merge_entries(overrides, &mut default_table);
                        };
                    }
                    global_table = Some(default_table);
                }
            }

            if global_table.is_some() || !config_dir.pop() {
                break;
            }
        }

        if global_table.is_none() {
            warn!("Config file {:?} not found", DEFAULT_CONFIG_FILENAME);
        }

        Self {
            config_path: config_dir,
            entries: global_table.unwrap_or_default(),
        }
    }

    fn merge_entries(apply: Table, dest: &mut Table) {
        apply.into_iter().for_each(|(section_name, section_table)| {
            if let Value::Table(section_table) = section_table {
                if let Some(dest_section) = dest
                    .entry(section_name)
                    .or_insert(Value::Table(Table::default()))
                    .as_table_mut()
                {
                    section_table.into_iter().for_each(|(key_name, value)| {
                        dest_section.insert(key_name, value); // override value in global table
                    });
                }
            }
        });
    }

    fn load_toml_table(filename: &Path) -> Option<Table> {
        std::fs::read_to_string(&filename)
            .map_err(|err| format!("Failed to read TOML file {:?}: {}", &filename, err))
            .and_then(|config_toml| {
                config_toml
                    .parse::<Value>()
                    .map_err(|err| format!("Failed to parse TOML file {:?}: {}", &filename, err))
                    .and_then(|table| {
                        table
                            .try_into::<Table>()
                            .map_err(|err| format!("Invalid TOML format {:?}: {}", &filename, err))
                    })
            })
            .map_err(|err| {
                warn!("{}", err);
                err
            })
            .ok()
    }

    fn find_table_entry<'a>(&'a self, property_name: &str) -> Option<&'a Value> {
        let mut parts = property_name.split('.');

        if let Some(first_part) = parts.next() {
            let mut node: &toml::Value = self.entries.get(first_part)?;

            for part in parts {
                if let toml::Value::Table(table) = node {
                    if let Some(value) = table.get(part) {
                        node = value;
                    } else {
                        warn!("Configs entry not found: {}", property_name);
                        return None;
                    }
                }
            }

            Some(node)
        } else {
            warn!("Configs entry not found: {}", property_name);
            None
        }
    }

    fn find_table_entry_mut<'a>(&'a mut self, property_name: &str) -> Option<&'a mut Value> {
        let mut parts = property_name.split('.');

        if let Some(first_part) = parts.next() {
            let mut node: &mut toml::Value = self.entries.get_mut(first_part)?;

            for part in parts {
                if let toml::Value::Table(table) = node {
                    if let Some(value) = table.get_mut(part) {
                        node = value;
                    } else {
                        warn!("Configs entry not found: {}", property_name);
                        return None;
                    }
                }
            }

            Some(node)
        } else {
            warn!("Configs entry not found: {}", property_name);
            None
        }
    }

    pub fn get<'de, T>(&self, key: &str) -> Option<T>
    where
        T: serde::Deserialize<'de>,
    {
        self.find_table_entry(key)
            .and_then(|value_entry| value_entry.clone().try_into::<T>().ok())
    }

    pub fn get_or<'a, T>(&'a self, key: &str, default_value: T) -> T
    where
        T: serde::Deserialize<'a>,
    {
        self.get(key).unwrap_or(default_value)
    }

    pub fn set<T>(&mut self, key: &str, value: T) -> bool
    where
        T: serde::Serialize,
    {
        self.find_table_entry_mut(key)
            .map(|entry_value| match Value::try_from(value) {
                Ok(new_value) if discriminant(&new_value) == discriminant(entry_value) => {
                    *entry_value = new_value;
                    Some(entry_value)
                }
                _ => None,
            })
            .is_some()
    }

    pub fn get_absolute_path(&self, key: &str) -> Option<PathBuf> {
        self.find_table_entry(key)
            .and_then(Value::as_str)
            .map(|str| self.config_path.join(str))
    }
}

lazy_static::lazy_static! {
    pub static ref CONFIGS : RwLock<Config> = RwLock::new(Config::new());
}

#[macro_export]
macro_rules! config_set {
    ($param:literal, $val:expr ) => {
        $crate::CONFIGS.write().unwrap().set($param, $val)
    };
}

#[macro_export]
macro_rules! config_get {
    ($param:literal) => {
        $crate::CONFIGS.read().unwrap().get($param)
    };
}

#[macro_export]
macro_rules! config_get_or {
    ($param:literal, $def:expr) => {
        $crate::CONFIGS.read().unwrap().get_or($param, $def)
    };
}

#[test]
fn test_config() {
    use std::collections::HashMap;

    let configs = Config::new();

    configs.get_absolute_path("editor_srv.project_dir").unwrap();

    let test_string: String = configs.get("test_config.test_string").unwrap();
    assert_eq!(test_string, "TestString");

    let test_bool: bool = configs.get("test_config.test_bool").unwrap();
    assert!(!test_bool);

    let test_int: i32 = configs.get("test_config.test_int").unwrap();
    assert_eq!(test_int, 1337);

    let test_float: f32 = configs.get("test_config.test_float").unwrap();
    assert!((test_float - 1337.1337f32).abs() < f32::EPSILON);

    let test_sub_config: i32 = configs.get("test_config.sub_config.test_nested").unwrap();
    assert_eq!(test_sub_config, 42);

    let test_config: HashMap<String, toml::Value> = configs.get("test_config").unwrap();
    assert_eq!(
        test_config,
        [
            (
                "test_string".to_string(),
                toml::Value::String("TestString".to_string())
            ),
            ("test_bool".to_string(), toml::Value::Boolean(false)),
            ("test_int".to_string(), toml::Value::Integer(1337)),
            ("test_float".to_string(), toml::Value::Float(1337.1337f64)),
            (
                "sub_config".to_string(),
                toml::Value::Table(
                    [("test_nested".to_string(), toml::Value::Integer(42),)]
                        .into_iter()
                        .collect()
                )
            ),
        ]
        .into_iter()
        .collect(),
    );
}

#[test]
fn test_singleton_configs() {
    let mut test_int: i32 = config_get!("test_config.test_int").unwrap();
    assert_eq!(test_int, 1337);

    assert!(config_set!("test_config.test_int", 1));

    test_int = config_get!("test_config.test_int").unwrap();
    assert_eq!(test_int, 1);
}
