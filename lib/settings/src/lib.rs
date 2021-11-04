use std::env;
use std::path::{Path, PathBuf};
use std::sync::{RwLock, RwLockReadGuard};
use toml::value::{Table, Value};

const DEFAULT_CONFIG_FILENAME: &str = "legion.toml";

pub struct Settings {
    config_path: PathBuf,
    table: RwLock<Table>,
}

impl Settings {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        // Search for the CONFIG file from the current exec direction,
        // walking up to the parents directory
        let mut config_dir = env::current_exe().unwrap();
        config_dir.pop();

        let mut table: Option<Table> = None;

        let file = Path::new(DEFAULT_CONFIG_FILENAME);
        loop {
            config_dir.push(file);

            if config_dir.is_file() {
                table = std::fs::read_to_string(&config_dir)
                    .map_err(|err| format!("Failed to read TOML file {:?}: {}", &config_dir, err))
                    .and_then(|config_toml| {
                        config_toml
                            .parse::<Value>()
                            .map_err(|err| {
                                format!("Failed to parse TOML file {:?}: {}", &config_dir, err)
                            })
                            .and_then(|table| {
                                table.try_into::<Table>().map_err(|err| {
                                    format!("Invalid TOML format {:?}: {}", &config_dir, err)
                                })
                            })
                    })
                    .map_err(|err| {
                        log::warn!("{}", err);
                        err
                    })
                    .ok();

                config_dir.pop();
                break;
            }
            if !(config_dir.pop() && config_dir.pop()) {
                break;
            }
        }

        if table.is_none() {
            log::warn!("Config file {:?} not found", file);
        }

        Self {
            config_path: config_dir,
            table: RwLock::new(table.unwrap_or_default()),
        }
    }

    fn find_table_entry<'a>(
        entries: &'a RwLockReadGuard<Table>,
        property_name: &str,
    ) -> Option<&'a Value> {
        property_name
            .split_once('.')
            .and_then(|(table_name, variable_name)| {
                entries
                    .get(table_name)
                    .and_then(|value| value.as_table())
                    .and_then(|table| table.get(variable_name))
            })
            .or_else(|| {
                log::warn!("Settings entry not found: {}", property_name);
                None
            })
    }

    pub fn get<'de, T>(&self, key: &str) -> Option<T>
    where
        T: serde::Deserialize<'de>,
    {
        Self::find_table_entry(&self.table.read().unwrap(), key)
            .and_then(|value_entry| value_entry.clone().try_into::<T>().ok())
    }

    pub fn get_absolute_path(&self, key: &str) -> Option<PathBuf> {
        Self::find_table_entry(&self.table.read().unwrap(), key)
            .and_then(|value_entry| value_entry.as_str())
            .map(|str| self.config_path.join(str))
    }
}

lazy_static::lazy_static! {
    pub static ref SETTINGS : Settings = Settings::new();
}

#[test]
fn test_settings() {
    let settings = Settings::new();

    settings
        .get_absolute_path("editor_srv.project_dir")
        .unwrap();

    let test_string: String = settings.get("test_settings.test_string").unwrap();
    assert_eq!(test_string, "TestString");

    let test_bool: bool = settings.get("test_settings.test_bool").unwrap();
    assert!(!test_bool);

    let test_int: i32 = settings.get("test_settings.test_int").unwrap();
    assert_eq!(test_int, 1337);

    let test_float: f32 = settings.get("test_settings.test_float").unwrap();
    assert!((test_float - 1337.1337f32).abs() < f32::EPSILON);
}
