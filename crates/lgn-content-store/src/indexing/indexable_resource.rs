use std::{fmt::Display, str::FromStr};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::{Error, Result};
use crate::{Identifier, Provider};

/// Represents a resource identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceIdentifier(pub(crate) Identifier);

impl Display for ResourceIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ResourceIdentifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.parse() {
            Ok(id) => Ok(Self(id)),
            Err(err) => Err(Error::InvalidResourceIdentifier(err)),
        }
    }
}

impl ResourceIdentifier {
    pub(crate) fn as_identifier(&self) -> &Identifier {
        &self.0
    }
}

/// A trait for resources that can be indexed.
#[async_trait]
pub trait IndexableResource {}

#[async_trait]
pub trait ResourceReader {
    async fn read_resource<R: IndexableResource + DeserializeOwned>(
        &self,
        id: &ResourceIdentifier,
    ) -> Result<R>;
}

#[async_trait]
impl ResourceReader for Provider {
    async fn read_resource<R: IndexableResource + DeserializeOwned>(
        &self,
        id: &ResourceIdentifier,
    ) -> Result<R> {
        let buf = self.read(&id.0).await?;

        Ok(rmp_serde::from_slice(&buf)?)
    }
}

#[async_trait]
pub trait ResourceWriter {
    async fn write_resource<R: IndexableResource + Serialize + Send + Sync>(
        &self,
        resource: &R,
    ) -> Result<ResourceIdentifier>;
}

#[async_trait]
impl ResourceWriter for Provider {
    async fn write_resource<R: IndexableResource + Serialize + Send + Sync>(
        &self,
        resource: &R,
    ) -> Result<ResourceIdentifier> {
        let buf = rmp_serde::to_vec(resource).unwrap();

        self.write(&buf)
            .await
            .map(ResourceIdentifier)
            .map_err(Into::into)
    }
}
