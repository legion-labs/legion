use std::{fmt::Display, str::FromStr};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{ContentReader, ContentReaderExt, ContentWriter, ContentWriterExt, Identifier, Result};

/// Represents a resource identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceIdentifier(pub(crate) Identifier);

impl Display for ResourceIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ResourceIdentifier {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self(s.parse()?))
    }
}

impl ResourceIdentifier {
    pub(crate) fn data_size(&self) -> usize {
        self.0.data_size()
    }

    pub(crate) fn as_identifier(&self) -> &Identifier {
        &self.0
    }
}

/// A trait for resources that can be indexed.
#[async_trait]
pub trait IndexableResource {}

#[async_trait]
pub trait ResourceReader: ContentReader + Send + Sync {
    async fn read_resource<R: IndexableResource + DeserializeOwned>(
        &self,
        id: &ResourceIdentifier,
    ) -> Result<R> {
        let buf = self.read_content(&id.0).await?;
        Ok(rmp_serde::from_slice(&buf)?)
    }
}

#[async_trait]
impl<T: ContentReader + Send + Sync> ResourceReader for T {}

#[async_trait]
pub trait ResourceWriter: ContentWriter + Send + Sync {
    async fn write_resource<R: IndexableResource + Serialize + Send + Sync>(
        &self,
        resource: &R,
    ) -> Result<ResourceIdentifier> {
        let buf = rmp_serde::to_vec(resource).unwrap();

        self.write_content(&buf).await.map(ResourceIdentifier)
    }
}

#[async_trait]
impl<T: ContentWriter + Send + Sync> ResourceWriter for T {}
