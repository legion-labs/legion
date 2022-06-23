use std::sync::Arc;

use strum_macros::{Display, EnumString};

mod inputs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentAddr(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumString, Display)]
pub enum Platform {
    PS5,
    //XSX,
    XB1,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumString, Display)]
pub enum Target {
    Client,
    Server,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumString, Display)]
pub enum Locale {
    English,
    French,
    Spanish,
    Japenese,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BuildParams {
    pub platform: Platform,
    pub target: Target,
    pub locale: Locale,
}

impl BuildParams {
    pub fn new(platform: Platform, target: Target, locale: Locale) -> Arc<Self> {
        Arc::new(Self {
            platform,
            target,
            locale,
        })
    }
}

impl Default for BuildParams {
    fn default() -> Self {
        Self {
            platform: Platform::PS5,
            target: Target::Client,
            locale: Locale::English,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompilerError {
    ParsingError,
}
