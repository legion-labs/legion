mod errors;

use std::path::Path;

use config::Config;
pub use errors::{Error, Result};

/// The default filename for configuration files.
pub static DEFAULT_FILENAME: &str = "legion.toml";

/// Load the configuration from all its various sources.
///
/// If a configuration value is set in different sources, the value from the
/// last read source will be used.
///
/// Namely, the configuration will be loaded from the following locations, in
/// order:
///
/// - `/etc/legion-labs/config.toml` on UNIX.
/// - `$XDG_CONFIG_HOME/legion-labs/config.toml` on UNIX.
/// - `$HOME/.config/legion-labs/config.toml` on UNIX.
/// - {FOLDERID_RoamingAppData}/legion-labs/legion.toml on Windows.
/// - Any `legion.toml` file in the current working directory, or one of its
/// parent directories, stopping as soon as a file is found.
/// - Environment variables, starting with `LGN`.
pub fn load_config() -> Result<Config> {
    let root = std::env::current_dir()?;

    load_config_from(root)
}

/// Load a configuration, using the specified root as a working-directory.
///
/// # Note
///
/// If root is a relative path, then the ancestors search will stop at the
/// relative root. While useful for tests, it's probably safer to invoke that
/// with an absolute path in any other case, always.
fn load_config_from(root: impl AsRef<Path>) -> Result<Config> {
    let mut builder = Config::builder();

    // On Unix, always read the system-wide configuration file first if it
    // exists.
    if cfg!(unix) {
        builder = builder
            .add_source(config::File::with_name("/etc/legion-labs/legion.toml").required(false));
    }

    // If we have usr configuration folder, try to read from it.
    if let Some(config_dir) = dirs::config_dir() {
        let config_file_path = config_dir.join("legion-labs").join(DEFAULT_FILENAME);
        builder = builder.add_source(config::File::from(config_file_path).required(false));
    }

    // Then, try to read the closest file we found.
    for dir in root.as_ref().ancestors() {
        let config_file_path = dir.join(DEFAULT_FILENAME);

        if std::fs::metadata(&config_file_path).is_ok() {
            builder = builder.add_source(config::File::from(config_file_path).required(false));
            break;
        }
    }

    // Finally, read from environment variables, starting with `LGN`.
    let config = builder
        .add_source(config::Environment::with_prefix("LGN").try_parsing(true))
        .build()
        .map_err(Error::Config)?;

    Ok(config)
}

#[cfg(test)]
mod tests {
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
        let config = load_config_from("src/fixtures/prod").unwrap();

        assert_eq!(
            "prod",
            config.get_string("lgn-config.tests.environment").unwrap()
        );
        assert!(config.get_string("lgn-config.tests.non-existing").is_err());
    }

    #[test]
    fn test_load_config_from_with_environment_variable_override() {
        // Lets set en environment variable, as an override.
        std::env::set_var("LGN_LGN-CONFIG.TESTS.ENVIRONMENT", "foo");
        let config = load_config_from("src/fixtures/prod").unwrap();
        std::env::remove_var("LGN_LGN-CONFIG.TESTS.ENVIRONMENT");

        assert_eq!(
            "foo",
            config.get_string("lgn-config.tests.environment").unwrap()
        );
    }

    #[test]
    fn test_load_config_from_with_struct() {
        let config = load_config_from("src/fixtures").unwrap();

        let my_config: MyConfig = config.get("lgn-config.tests.my_config").unwrap();

        assert_eq!(true, my_config.my_bool);
        assert_eq!(42, my_config.my_int);
        assert_eq!(3.14, my_config.my_float);
        assert_eq!(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            my_config.my_list
        );
    }
}
