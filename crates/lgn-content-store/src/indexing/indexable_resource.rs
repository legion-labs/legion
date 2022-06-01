use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

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

#[async_trait]
pub trait ResourceExists {
    async fn resource_exists(&self, id: &ResourceIdentifier) -> Result<bool>;
}

#[async_trait]
impl ResourceExists for Provider {
    async fn resource_exists(&self, id: &ResourceIdentifier) -> Result<bool> {
        self.exists(id.as_identifier()).await.map_err(Into::into)
    }
}

/// A trait for resources that can be indexed.
pub trait IndexableResource {}

#[async_trait]
pub trait ResourceReader {
    async fn read_resource<R: IndexableResource + DeserializeOwned>(
        &self,
        id: &ResourceIdentifier,
    ) -> Result<R>;

    async fn read_resource_as_bytes(&self, id: &ResourceIdentifier) -> Result<Vec<u8>>;
}

#[async_trait]
impl ResourceReader for Provider {
    async fn read_resource<R: IndexableResource + DeserializeOwned>(
        &self,
        id: &ResourceIdentifier,
    ) -> Result<R> {
        let buf = self.read_resource_as_bytes(id).await?;

        Ok(rmp_serde::from_slice(&buf)?)
    }

    async fn read_resource_as_bytes(&self, id: &ResourceIdentifier) -> Result<Vec<u8>> {
        self.read(&id.0).await.map_err(Into::into)
    }
}

#[async_trait]
pub trait ResourceWriter {
    async fn write_resource<R: IndexableResource + Serialize + Send + Sync>(
        &self,
        resource: &R,
    ) -> Result<ResourceIdentifier>;

    async fn unwrite_resource(&self, id: &ResourceIdentifier) -> Result<()>;
}

#[async_trait]
impl ResourceWriter for Provider {
    async fn write_resource<R: IndexableResource + Serialize + Send + Sync>(
        &self,
        resource: &R,
    ) -> Result<ResourceIdentifier> {
        let buf = rmp_serde::to_vec(resource).unwrap();

        self.write_resource_from_bytes(&buf).await
    }

    async fn write_resource_from_bytes(&self, data: &[u8]) -> Result<ResourceIdentifier> {
        self.write(data)
            .await
            .map(ResourceIdentifier)
            .map_err(Into::into)
    }

    async fn unwrite_resource(&self, id: &ResourceIdentifier) -> Result<()> {
        self.unwrite(id.as_identifier()).await.map_err(Into::into)
    }
}

pub struct ResourceByteWriter<'a>(&'a [u8]);

impl<'a> IndexableResource for ResourceByteWriter<'a> {}

impl<'a> ResourceByteWriter<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }
}

impl<'a> serde::Serialize for ResourceByteWriter<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.0)
    }
}

pub struct ResourceByteReader(Vec<u8>);

impl IndexableResource for ResourceByteReader {}

impl ResourceByteReader {
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

struct ResourceByteReaderVisitor;

impl<'de> serde::de::Visitor<'de> for ResourceByteReaderVisitor {
    type Value = ResourceByteReader;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ResourceByteReader(v.to_vec()))
    }
}

impl<'de> serde::Deserialize<'de> for ResourceByteReader {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(ResourceByteReaderVisitor)
    }
}
