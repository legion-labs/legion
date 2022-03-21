mod errors;

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use std::path::Path;

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
pub fn load_config() -> Result<Figment> {
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
fn load_config_from(root: impl AsRef<Path>) -> Result<Figment> {
    let mut config = Figment::new();

    // On Unix, always read the system-wide configuration file first if it
    // exists.
    if cfg!(unix) {
        config = config.merge(Toml::file("/etc/legion-labs/legion.toml"));
    }

    // If we have usr configuration folder, try to read from it.
    if let Some(config_dir) = dirs::config_dir() {
        let config_file_path = config_dir.join("legion-labs").join(DEFAULT_FILENAME);
        config = config.merge(Toml::file(config_file_path));
    }

    // Then, try to read the closest file we found.
    for dir in root.as_ref().ancestors() {
        let config_file_path = dir.join(DEFAULT_FILENAME);

        if std::fs::metadata(&config_file_path).is_ok() {
            config = config.merge(Toml::file(config_file_path));
            break;
        }
    }

    // Finally, read from environment variables, starting with `LGN`.
    let config = config.merge(Env::prefixed("LGN_"));

    Ok(config)
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
        let config = load_config_from("src/fixtures/prod").unwrap();

        assert_eq!(
            "prod",
            config
                .extract_inner::<String>("lgn-config.tests.environment")
                .unwrap()
        );
        assert!(config
            .extract_inner::<String>("lgn-config.tests.non-existing")
            .is_err());
    }

    #[test]
    fn test_load_config_from_with_environment_variable_override() {
        Jail::expect_with(|jail| {
            // Lets set en environment variable, as an override.
            jail.set_env("LGN_LGN-CONFIG.TESTS.ENVIRONMENT", "foo");
            let config = load_config_from("src/fixtures/prod").unwrap();

            assert_eq!(
                "foo",
                config
                    .extract_inner::<String>("lgn-config.tests.environment")
                    .unwrap()
            );

            Ok(())
        });
    }

    #[test]
    fn test_load_config_from_with_struct() {
        let config = load_config_from("src/fixtures").unwrap();

        let my_config: MyConfig = config.extract_inner("lgn-config.tests.my_config").unwrap();

        assert_eq!(true, my_config.my_bool);
        assert_eq!(42, my_config.my_int);
        assert_eq!(3.14, my_config.my_float);
        assert_eq!(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            my_config.my_list
        );
    }

    #[test]
    fn test_load_config_from_relative_path_buf() {
        let config = load_config_from("src/fixtures/prod").unwrap();

        let path = config
            .extract_inner::<RelativePathBuf>("lgn-config.tests.avatar")
            .unwrap();

        assert_eq!("../images/avatar.png", path.original().to_str().unwrap());

        let cwd = std::env::current_dir().unwrap().join("src/fixtures/prod");
        assert_eq!(cwd.join("../images/avatar.png"), path.relative());
    }
}
