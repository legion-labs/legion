use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use async_trait::async_trait;
use http_body::Body;
use lgn_content_store_proto::content_store_client::ContentStoreClient;
use lgn_tracing::async_span_scope;
use tokio::sync::Mutex;
use tonic::codegen::StdError;

use super::{AliasReader, AliasWriter, Error, Result};
use crate::{DataSpace, Identifier};

/// A `GrpcAliasProvider` is a provider that delegates to a `gRPC` service.
#[derive(Debug, Clone)]
pub struct GrpcAliasProvider<C> {
    client: Arc<Mutex<ContentStoreClient<C>>>,
    data_space: DataSpace,
}

impl<C> GrpcAliasProvider<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    pub async fn new(grpc_client: C, data_space: DataSpace) -> Self {
        let client = Arc::new(Mutex::new(ContentStoreClient::new(grpc_client)));

        Self { client, data_space }
    }
}

impl<C> Display for GrpcAliasProvider<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "gRPC client (data space: {})", self.data_space)
    }
}

#[async_trait]
impl<C> AliasReader for GrpcAliasProvider<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send + Debug,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    async fn resolve_alias(&self, key: &[u8]) -> Result<Identifier> {
        async_span_scope!("GrpcAliasProvider::resolve_alias");

        let req = lgn_content_store_proto::ResolveAliasRequest {
            data_space: self.data_space.to_string(),
            key: key.into(),
        };

        let resp = self
            .client
            .lock()
            .await
            .resolve_alias(req)
            .await
            .map_err(|err| anyhow::anyhow!("gRPC request failed: {}", err))?
            .into_inner();

        if resp.id.is_empty() {
            Err(Error::AliasNotFound(key.into()))
        } else {
            Ok(resp.id.parse()?)
        }
    }
}

#[async_trait]
impl<C> AliasWriter for GrpcAliasProvider<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send + Debug + 'static,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    async fn register_alias(&self, key: &[u8], id: &Identifier) -> Result<Identifier> {
        async_span_scope!("GrpcAliasProvider::register_alias");

        let req = lgn_content_store_proto::RegisterAliasRequest {
            data_space: self.data_space.to_string(),
            key: key.into(),
            id: id.to_string(),
        };

        let resp = self
            .client
            .lock()
            .await
            .register_alias(req)
            .await
            .map_err(|err| anyhow::anyhow!("gRPC request failed: {}", err))?
            .into_inner();

        if resp.newly_registered {
            Ok(resp.id.parse()?)
        } else {
            Err(Error::AliasAlreadyExists(key.into()))
        }
    }
}
