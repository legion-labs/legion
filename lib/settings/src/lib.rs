use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{RwLock, RwLockReadGuard};
use toml::value::{Table, Value};

const DEFAULT_CONFIG_FILENAME: &str = "legionapp.toml";

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
                if let Ok(mut file) = File::open(&config_dir) {
                    let mut config_toml = String::new();
                    if file.read_to_string(&mut config_toml).is_ok() {
                        let parsed_value = config_toml.parse::<Value>().unwrap_or_else(|err| {
                            log::warn!("Failed to parse TOML {:?}: {}", &config_dir, err);
                            Table::default().into()
                        });
                        if let Ok(converted_table) = parsed_value.try_into::<Table>() {
                            table = Some(converted_table);
                        } else {
                            log::warn!("Invalid TOML format {:?}. Not a table", &config_dir);
                        }
                    }
                    config_dir.pop();
                }
                break;
            }
            if !(config_dir.pop() && config_dir.pop()) {
                break;
            }
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
        if let Some((table_name, variable_name)) = property_name.split_once('.') {
            if let Some(value) = entries.get(table_name) {
                if let Some(table) = value.as_table() {
                    return table.get(variable_name);
                }
            }
        }
        log::warn!("Settings entry not found: {}", property_name);
        None
    }

    pub fn get<'de, T>(&self, key: &str) -> Option<T>
    where
        T: serde::Deserialize<'de>,
    {
        if let Some(value_entry) = Self::find_table_entry(&self.table.read().unwrap(), key) {
            if let Ok(value) = value_entry.clone().try_into() {
                return Some(value);
            }
        }
        None
    }

    pub fn get_absolute_path(&self, key: &str) -> Option<PathBuf> {
        if let Some(value) = Self::find_table_entry(&self.table.read().unwrap(), key) {
            if let Some(str) = value.as_str() {
                return Some(self.config_path.join(str));
            }
        }
        None
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
