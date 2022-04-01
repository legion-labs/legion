//! Configuration.
//!
//! This crate provides a simple configuration system.

mod errors;
mod rich_pathbuf;

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use std::path::{Path, PathBuf};

pub use errors::{Error, Result};
pub use rich_pathbuf::RichPathBuf;

/// The type to use for relative paths in configurations.
pub use figment::value::magic::RelativePathBuf;

/// The default filename for configuration files.
pub static DEFAULT_FILENAME: &str = "legion.toml";

#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) figment: Figment,
}

lazy_static::lazy_static! {
    pub static ref CONFIG : Config = Config::load().expect("failed to the load the configuration");
}

/// Get the value specified by the key.
///
/// If the value does not exist, None is returned.
///
/// # Efficiency
///
/// This method is provided for convenience. If you intend to read several
/// configuration values, it is more efficient to either read a struct or
/// instantiate a `Config` and use the `get` method on the struct.
///
/// # Errors
///
/// If any error occurs, including the specified key not existing in the
/// configuration, it is returned.
pub fn get<'de, T>(key: &str) -> Result<Option<T>>
where
    T: serde::Deserialize<'de>,
{
    (*CONFIG).get(key)
}

/// Get the value specified by the key or a specified default value if it is
/// not found.
///
/// If the value does not exist, the specified default value is returned.
///
/// # Efficiency
///
/// This method is provided for convenience. If you intend to read several
/// configuration values, it is more efficient to either read a struct or
/// instantiate a `Config` and use the `get_or` method on the struct.
///
/// # Errors
///
/// If any other error occurs, it is returned.
pub fn get_or<'de, T>(key: &str, default: T) -> Result<T>
where
    T: serde::Deserialize<'de>,
{
    (*CONFIG).get_or(key, default)
}

/// Get the value specified by the key or builds a default value by calling
/// the specified function if the key is not found.
///
/// # Efficiency
///
/// This method is provided for convenience. If you intend to read several
/// configuration values, it is more efficient to either read a struct or
/// instantiate a `Config` and use the `get_or_else` method on the struct.
///
/// # Errors
///
/// If any other error occurs, it is returned.
pub fn get_or_else<'de, T, F: FnOnce() -> T>(key: &str, f: F) -> Result<T>
where
    T: serde::Deserialize<'de>,
{
    (*CONFIG).get_or_else(key, f)
}

/// Get the value specified by the key or a default value if it is not
/// found.
///
/// If the value does not exist, the default value is returned.
///
/// # Efficiency
///
/// This method is provided for convenience. If you intend to read several
/// configuration values, it is more efficient to either read a struct or
/// instantiate a `Config` and use the `get_or_default` method on the
/// struct.
///
/// # Errors
///
/// If any other error occurs, it is returned.
pub fn get_or_default<'de, T>(key: &str) -> Result<T>
where
    T: serde::Deserialize<'de> + Default,
{
    (*CONFIG).get_or_default(key)
}

/// Get the absolute path at the specified key.
///
/// If the specified path is relative, it will be resolved relative to its
/// containing configuration file, or the current working directory if the
/// value does not come from a file.
///
/// # Efficiency
///
/// This method is provided for convenience. If you intend to read several
/// configuration values, it is more efficient to either read a struct or
/// instantiate a `Config` and use the `get_absolute_path` method on the
/// struct.
///
/// # Errors
///
/// If any error occurs, including the specified key not existing in the
/// configuration, it is returned.
pub fn get_absolute_path(key: &str) -> Result<Option<PathBuf>> {
    (*CONFIG).get_absolute_path(key)
}

/// Get the absolute path at the specified key or the specified default.
///
/// If the specified path is relative, it will be resolved relative to its
/// containing configuration file, or the current working directory if the
/// value does not come from a file.
///
/// # Efficiency
///
/// This method is provided for convenience. If you intend to read several
/// configuration values, it is more efficient to either read a struct or
/// instantiate a `Config` and use the `get_absolute_path` method on the
/// struct.
///
/// # Errors
///
/// If any error occurs, including the specified key not existing in the
/// configuration, it is returned.
pub fn get_absolute_path_or(key: &str, default: PathBuf) -> Result<PathBuf> {
    (*CONFIG).get_absolute_path_or(key, default)
}

impl Config {
    /// Create a configuration from a TOML string.
    ///
    /// Useful for tests mostly.
    pub fn from_toml(toml: &str) -> Self {
        let figment = Figment::new().merge(Toml::string(toml));
        Self { figment }
    }

    /// Load the configuration from all its various sources.
    ///
    /// If a configuration value is set in different sources, the value from the
    /// last read source will be used.
    ///
    /// Namely, the configuration will be loaded from the following locations, in
    /// order:
    ///
    /// - `/etc/legion-labs/config.toml` on UNIX.
    /// - Any `legion.toml` file in the current binary directory, or one of its
    /// parent directories, stopping as soon as a file is found.
    /// - Any `legion.toml` file in the current working directory, or one of its
    /// parent directories, stopping as soon as a file is found. If the first
    /// found file was already read as part of the previous lookup, it is not
    /// read again.
    /// - `$XDG_CONFIG_HOME/legion-labs/config.toml` on UNIX.
    /// - `$HOME/.config/legion-labs/config.toml` on UNIX.
    /// - %APPDATA%/legion-labs/legion.toml on Windows.
    /// - Any file specified in the `LGN_CONFIG` environment variable.
    /// - Environment variables, starting with `LGN_`.
    ///
    /// # Errors
    ///
    /// If the configuration cannot be loaded, an error is returned.
    pub fn load() -> Result<Self> {
        let path = std::env::current_dir()?;

        Self::load_with_current_directory(path)
    }

    /// Load a configuration, using the specified root as the current directory.
    ///
    /// See `load()` for more information.
    ///
    /// # Note
    ///
    /// If root is a relative path, then the ancestors search will stop at the
    /// relative root. While useful for tests, it's probably safer to invoke that
    /// with an absolute path in any other case, always.
    ///
    /// # Errors
    ///
    /// If the configuration cannot be loaded, an error is returned.
    pub fn load_with_current_directory(path: impl AsRef<Path>) -> Result<Self> {
        let mut figment = Figment::new();

        // On Unix, always read the system-wide configuration file first if it
        // exists.
        if cfg!(unix) {
            figment = figment.merge(Toml::file(format!("/etc/legion-labs/{}", DEFAULT_FILENAME)));
        }

        // Starting with the current binary directory, walk up to the root,
        // stopping as soon as we find a configuration file.
        let binary_path = std::env::current_exe()?;

        let mut known_path = None;

        for dir in binary_path.parent().unwrap().ancestors() {
            let config_file_path = dir.join(DEFAULT_FILENAME);

            if std::fs::metadata(&config_file_path).is_ok() {
                figment = figment.merge(Toml::file(&config_file_path));
                known_path = Some(config_file_path);
                break;
            }
        }

        // Then, try to read the closest file we found.
        for dir in path.as_ref().ancestors() {
            let config_file_path = dir.join(DEFAULT_FILENAME);

            if std::fs::metadata(&config_file_path).is_ok() {
                // If we already loaded that file, do not reload it.
                if let Some(known_path) = known_path {
                    if config_file_path == known_path {
                        break;
                    }
                }

                figment = figment.merge(Toml::file(config_file_path));
                break;
            }
        }

        // If we have an user configuration folder, try to read from it.
        if let Some(config_dir) = dirs::config_dir() {
            let config_file_path = config_dir.join("legion-labs").join(DEFAULT_FILENAME);
            figment = figment.merge(Toml::file(config_file_path));
        }

        // If a specific configuration file was specified, try to read it.
        if let Some(config_file_path) = std::env::var_os("LGN_CONFIG") {
            figment = figment.merge(Toml::file(config_file_path));
        }

        // Finally, read from environment variables, starting with `LGN`.
        let figment = figment.merge(Env::prefixed("LGN_"));

        Ok(Self { figment })
    }

    /// Override this configuration with another one.
    pub fn override_with(&mut self, other: Self) {
        let figment = std::mem::take(&mut self.figment);
        self.figment = figment.merge(other.figment);
    }

    /// Get the value specified by the key.
    ///
    /// If the value does not exist, None is returned.
    ///
    /// # Errors
    ///
    /// If any error occurs, including the specified key not existing in the
    /// configuration, it is returned.
    pub fn get<'de, T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::Deserialize<'de>,
    {
        match self.figment.extract_inner(key) {
            Ok(value) => Ok(Some(value)),
            Err(err) => match &err.kind {
                figment::error::Kind::MissingField(missing_key) => {
                    if key == missing_key {
                        Ok(None)
                    } else {
                        Err(Box::new(err).into())
                    }
                }
                _ => Err(Box::new(err).into()),
            },
        }
    }

    /// Get the value specified by the key or a specified default value if it is
    /// not found.
    ///
    /// If the value does not exist, the specified default value is returned.
    ///
    /// # Errors
    ///
    /// If any other error occurs, it is returned.
    pub fn get_or<'de, T>(&self, key: &str, default: T) -> Result<T>
    where
        T: serde::Deserialize<'de>,
    {
        self.get(key).map(|value| value.unwrap_or(default))
    }

    /// Get the value specified by the key or builds a default value by calling
    /// the specified function if the key is not found.
    ///
    /// # Errors
    ///
    /// If any other error occurs, it is returned.
    pub fn get_or_else<'de, T, F>(&self, key: &str, f: F) -> Result<T>
    where
        T: serde::Deserialize<'de>,
        F: FnOnce() -> T,
    {
        self.get(key).map(|value| value.unwrap_or_else(f))
    }

    /// Get the value specified by the key or a default value if it is not
    /// found.
    ///
    /// If the value does not exist, the default value is returned.
    ///
    /// # Errors
    ///
    /// If any other error occurs, it is returned.
    pub fn get_or_default<'de, T>(&self, key: &str) -> Result<T>
    where
        T: serde::Deserialize<'de> + Default,
    {
        self.get(key).map(Option::unwrap_or_default)
    }

    /// Get the absolute path at the specified key.
    ///
    /// If the specified path is relative, it will be resolved relative to its
    /// containing configuration file, or the current working directory if the
    /// value does not come from a file.
    ///
    /// # Errors
    ///
    /// If any error occurs, including the specified key not existing in the
    /// configuration, it is returned.
    pub fn get_absolute_path(&self, key: &str) -> Result<Option<PathBuf>> {
        if let Some(path) = self.get::<RelativePathBuf>(key)? {
            let path = path.relative();

            Ok(Some(if path.is_absolute() {
                path
            } else {
                std::env::current_dir()?.join(path)
            }))
        } else {
            Ok(None)
        }
    }

    /// Get the absolute path at the specified key or the specified default.
    ///
    /// If the specified path is relative, it will be resolved relative to its
    /// containing configuration file, or the current working directory if the
    /// value does not come from a file.
    ///
    /// # Errors
    ///
    /// If any error occurs, including the specified key not existing in the
    /// configuration, it is returned.
    pub fn get_absolute_path_or(&self, key: &str, default: PathBuf) -> Result<PathBuf> {
        let path = self
            .get_or_else::<RelativePathBuf, _>(key, || RelativePathBuf::from(default))?
            .relative();

        Ok(if path.is_absolute() {
            path
        } else {
            std::env::current_dir()?.join(path)
        })
    }
}

#[cfg(test)]
mod tests {
    use figment::{value::magic::RelativePathBuf, Jail};
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    struct MyConfig {
        my_bool: bool,
        my_int: i64,
        my_float: f64,
        my_list: Vec<String>,
    }

    #[test]
    fn test_load_config_from() {
        Jail::expect_with(|_| {
            let config = Config::load_with_current_directory(
                &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/fixtures/prod"),
            )
            .unwrap();

            assert_eq!(
                Some("prod"),
                config
                    .get::<String>("lgn-config.tests.environment")
                    .unwrap()
                    .as_deref()
            );
            assert!(config
                .get::<String>("lgn-config.tests.non-existing")
                .unwrap()
                .is_none());
            assert_eq!(
                "",
                config
                    .get_or_default::<String>("lgn-config.tests.non-existing")
                    .unwrap()
            );

            Ok(())
        });
    }

    #[test]
    fn test_load_config_from_with_environment_variable_override() {
        Jail::expect_with(|jail| {
            // Lets set en environment variable, as an override.
            jail.set_env("LGN_LGN-CONFIG.TESTS.ENVIRONMENT", "foo");
            let config = Config::load_with_current_directory(
                &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/fixtures/prod"),
            )
            .unwrap();

            assert_eq!(
                Some("foo"),
                config
                    .get::<String>("lgn-config.tests.environment")
                    .unwrap()
                    .as_deref()
            );

            Ok(())
        });
    }

    #[test]
    fn test_load_config_from_with_struct() {
        Jail::expect_with(|_| {
            let config = Config::load_with_current_directory(
                &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/fixtures"),
            )
            .unwrap();

            let my_config: MyConfig = config.get("lgn-config.tests.my_config").unwrap().unwrap();

            assert!(my_config.my_bool);
            assert_eq!(42, my_config.my_int);
            //assert_eq!(1.23, my_config.my_float);
            assert_eq!(
                vec!["a".to_string(), "b".to_string(), "c".to_string()],
                my_config.my_list
            );
            Ok(())
        });
    }

    #[test]
    fn test_load_config_from_relative_path_buf() {
        Jail::expect_with(|_| {
            let base = &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/fixtures/prod");
            let config = Config::load_with_current_directory(&base).unwrap();

            let path = config
                .get::<RelativePathBuf>("lgn-config.tests.avatar")
                .unwrap()
                .unwrap();

            assert_eq!("../images/avatar.png", path.original().to_str().unwrap());
            assert_eq!(base.join("../images/avatar.png"), path.relative());

            // Test reading a relative path buf nested in a configuration.
            #[derive(Deserialize, Serialize, Debug)]
            struct MyConfig {
                avatar: RelativePathBuf,
            }

            let cfg = config.get::<MyConfig>("lgn-config.tests").unwrap().unwrap();

            assert_eq!(
                "../images/avatar.png",
                cfg.avatar.original().to_str().unwrap()
            );
            assert_eq!(base.join("../images/avatar.png"), cfg.avatar.relative());

            Ok(())
        });
    }
}
