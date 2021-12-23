use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct CargoConfig {
    pub sccache: Option<bool>,
}

pub struct Config {
    pub(crate) cargo_config: CargoConfig,
}
impl Config {
    pub(crate) fn cargo_config(&self) -> &CargoConfig {
        &self.cargo_config
    }
}
