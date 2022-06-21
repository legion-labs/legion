use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use crate::api::content_store::{
    client::{
        Client, RegisterAliasRequest, RegisterAliasResponse, ResolveAliasRequest,
        ResolveAliasResponse,
    },
    AliasKey,
};
use async_trait::async_trait;
use http::Uri;
use lgn_governance::types::SpaceId;
use lgn_tracing::async_span_scope;

use super::{AliasReader, AliasWriter, Error, Result};
use crate::{DataSpace, Identifier};

/// A `ApiAliasProvider` is a provider that delegates to a `gRPC` service.
#[derive(Debug, Clone)]
pub struct ApiAliasProvider<C> {
    client: Arc<Client<C>>,
    space_id: SpaceId,
    data_space: DataSpace,
}

impl<C> ApiAliasProvider<C> {
    pub async fn new(client: C, base_url: Uri, data_space: DataSpace) -> Self {
        let client = Arc::new(Client::new(client, base_url));

        Self {
            client,
            space_id: "default".parse().unwrap(),
            data_space,
        }
    }
}

impl<C> Display for ApiAliasProvider<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "open api client (data space: {})", self.data_space)
    }
}

#[async_trait]
impl<C, ResBody> AliasReader for ApiAliasProvider<C>
where
    C: tower::Service<http::Request<hyper::Body>, Response = http::Response<ResBody>>
        + Clone
        + Send
        + Sync
        + Debug
        + 'static,
    C::Error: Into<lgn_online::client::Error>,
    C::Future: Send,
    ResBody: hyper::body::HttpBody + Send,
    ResBody::Data: Send,
    ResBody::Error: std::error::Error,
{
    async fn resolve_alias(&self, key: &[u8]) -> Result<Identifier> {
        async_span_scope!("ApiAliasProvider::resolve_alias");

        let req = ResolveAliasRequest {
            space_id: self.space_id.clone().into(),
            data_space: self.data_space.clone().into(),
            alias_key: AliasKey(key.into()),
        };

        let resp = self
            .client
            .resolve_alias(req)
            .await
            .map_err(|err| anyhow::anyhow!("request failed: {}", err))?;

        match resp {
            ResolveAliasResponse::Status200 { body, .. } => Ok(body.id.try_into()?),
            ResolveAliasResponse::Status404 { .. } => Err(Error::AliasNotFound(key.into())),
        }
    }
}

#[async_trait]
impl<C, ResBody> AliasWriter for ApiAliasProvider<C>
where
    C: tower::Service<http::Request<hyper::Body>, Response = http::Response<ResBody>>
        + Clone
        + Send
        + Sync
        + Debug
        + 'static,
    C::Error: Into<lgn_online::client::Error>,
    C::Future: Send,
    ResBody: hyper::body::HttpBody + Send,
    ResBody::Data: Send,
    ResBody::Error: std::error::Error,
{
    async fn register_alias(&self, key: &[u8], id: &Identifier) -> Result<Identifier> {
        async_span_scope!("ApiAliasProvider::register_alias");

        let req = RegisterAliasRequest {
            space_id: self.space_id.clone().into(),
            data_space: self.data_space.clone().into(),
            alias_key: AliasKey(key.into()),
            content_id: id.into(),
        };

        let resp = self
            .client
            .register_alias(req)
            .await
            .map_err(|err| anyhow::anyhow!("request failed: {}", err))?;

        match resp {
            RegisterAliasResponse::Status201 { body, .. } => Ok(body.id.try_into()?),
            RegisterAliasResponse::Status409 { .. } => Err(Error::AliasAlreadyExists(key.into())),
        }
    }
}
