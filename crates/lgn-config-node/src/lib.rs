//! Loads client configuration from legion.toml files and return
//! a whole struct ready to use with Node's env and Vite.

use std::collections::HashMap;

use napi::{
    bindgen_prelude::{Error, Result},
    Env, JsObject,
};
use napi_derive::napi;

/// Get the value specified by the key.
///
/// If the value does not exist, None is returned.
///
/// # Errors
///
/// If any error occurs, including the specified key not existing in the
/// configuration, it is returned.
#[napi]
#[allow(clippy::needless_pass_by_value)]
pub fn get(key: String) -> Result<Option<serde_json::Value>> {
    let value: Option<serde_json::Value> =
        lgn_config::get(&key).map_err(|error| Error::from_reason(error.to_string()))?;

    Ok(value)
}

/// Get the value specified by the key or a specified default value if it is
/// not found.
///
/// If the value does not exist, the specified default value is returned.
///
/// # Errors
///
/// If any other error occurs, it is returned.
#[napi]
#[allow(clippy::needless_pass_by_value)]
pub fn get_or(key: String, default_value: serde_json::Value) -> Result<serde_json::Value> {
    let value = lgn_config::get_or(&key, default_value)
        .map_err(|error| Error::from_reason(error.to_string()))?;

    Ok(value)
}

/// Takes a [`HashMap`] which keys are environment variable names and values are
/// config keys and return another [`HashMap`] which values have been resolved.
///
/// # Example
///
/// ```ts
/// getAll({
///   SOMETHING_USEFUL: "key.subkey",
///   ANOTHER_GREAT_THING: "another.deeper.key",
/// });
/// ```
///
/// Given the config files are defined accordingly the above will return:
///
/// ```json
/// {
///   "SOMETHING_USEFUL": "I found this useful value in the config file(s)",
///   "ANOTHER_GREAT_THING": "An this value too"
/// }
/// ```
///
/// If a key is not found, the value set is "`null`"
///
/// # Errors
///
/// If any error occurs, including the specified key not existing in the
/// configuration, it is returned.
#[napi]
#[allow(clippy::needless_pass_by_value, clippy::implicit_hasher)]
pub fn get_all(
    keys: HashMap<String, String>,
) -> Result<HashMap<String, Option<serde_json::Value>>> {
    let values = keys
        .into_iter()
        .map(|(key, value)| Ok((key, get(value)?)))
        .collect::<Result<_>>()?;

    Ok(values)
}

/// Works similarly to the [`get_all`] function but will immediately set the found variables
/// in the global `process.env` object.
///
/// If a key is not found, the value is ignored.
///
/// # Errors
///
/// If any error occurs, including the specified key not existing in the
/// configuration, it is returned.
#[napi]
#[allow(clippy::needless_pass_by_value, clippy::implicit_hasher)]
pub fn load_all(env: Env, keys: HashMap<String, String>) -> Result<()> {
    let mut node_env = env
        .get_global()?
        .get_named_property::<JsObject>("process")?
        .get_named_property::<JsObject>("env")?;

    keys.into_iter().try_for_each(|(key, value)| {
        let value = get(value)?.and_then(|value| value.as_str().map(str::to_string));

        if let Some(value) = value {
            node_env.set_named_property(&key, env.create_string(&value)?)?;
        };

        Ok::<(), Error>(())
    })?;

    Ok(())
}
