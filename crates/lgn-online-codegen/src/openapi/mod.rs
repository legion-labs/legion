use std::fs;

use super::errors::Result;

#[derive(Debug)]
pub struct Spec {
    pub info: Info,
}

#[derive(Debug)]
pub struct Info {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

impl Spec {
    pub fn from_yaml_file(path: &str) -> Result<Self> {
        let file = fs::File::open(path)?;
        let openapi: openapiv3::OpenAPI = serde_yaml::from_reader(&file)?;
        Ok(openapi.into())
    }
}

impl From<openapiv3::OpenAPI> for Spec {
    fn from(openapi: openapiv3::OpenAPI) -> Self {
        Self {
            info: openapi.info.into(),
        }
    }
}

impl From<openapiv3::Info> for Info {
    fn from(info: openapiv3::Info) -> Self {
        Self {
            title: info.title,
            version: info.version,
            description: info.description,
        }
    }
}
