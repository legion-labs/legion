use crate::config::{CargoConfig, Config};
use crate::Result;

pub struct Context {
    config: Config,
}

impl Context {
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: Config {
                cargo_config: CargoConfig { sccache: None },
            },
        })
    }
    pub fn config(&self) -> &Config {
        &self.config
    }
}
