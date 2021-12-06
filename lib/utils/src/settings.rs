use std::env;
use std::mem::discriminant;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use toml::value::{Table, Value};

const DEFAULT_CONFIG_FILENAME: &str = "legion.toml";
const LOCAL_CONFIG_FILENAME: &str = "legion_local.toml";

pub struct Settings {
    config_path: PathBuf,
    entries: Table,
}

impl Settings {
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
            log::warn!("Config file {:?} not found", DEFAULT_CONFIG_FILENAME);
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
                log::warn!("{}", err);
                err
            })
            .ok()
    }

    fn find_table_entry<'a>(&'a self, property_name: &str) -> Option<&'a Value> {
        property_name
            .split_once('.')
            .and_then(|(table_name, variable_name)| {
                self.entries
                    .get(table_name)
                    .and_then(Value::as_table)
                    .and_then(|table| table.get(variable_name))
            })
            .or_else(|| {
                log::warn!("Settings entry not found: {}", property_name);
                None
            })
    }

    fn find_table_entry_mut<'a>(&'a mut self, property_name: &str) -> Option<&'a mut Value> {
        property_name
            .split_once('.')
            .and_then(|(table_name, variable_name)| {
                self.entries
                    .get_mut(table_name)
                    .and_then(Value::as_table_mut)
                    .and_then(|table| table.get_mut(variable_name))
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
    pub static ref SETTINGS : RwLock<Settings> = RwLock::new(Settings::new());
}

#[macro_export]
macro_rules! setting_set {
    ($param:literal, $val:expr ) => {
        $crate::SETTINGS.write().unwrap().set($param, $val)
    };
}

#[macro_export]
macro_rules! setting_get {
    ($param:literal) => {
        $crate::SETTINGS.read().unwrap().get($param)
    };
}

#[macro_export]
macro_rules! setting_get_or {
    ($param:literal, $def:expr) => {
        $crate::SETTINGS.read().unwrap().get_or($param, $def)
    };
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

#[test]
fn test_singleton_settings() {
    let mut test_int: i32 = setting_get!("test_settings.test_int").unwrap();
    assert_eq!(test_int, 1337);

    assert!(setting_set!("test_settings.test_int", 1));

    test_int = setting_get!("test_settings.test_int").unwrap();
    assert_eq!(test_int, 1);
}
