use std::{collections::HashMap, sync::Arc};

use crate::api::content_store::{
    server::{
        GetContentWriterRequest, GetContentWriterResponse, ReadContentRequest, ReadContentResponse,
        RegisterAliasRequest, RegisterAliasResponse, ResolveAliasRequest, ResolveAliasResponse,
        WriteContentRequest, WriteContentResponse,
    },
    Api, ContentId, GetContentWriter200Response, Origin, RegisterAlias201Response,
    ResolveAlias200Response, Url, WriteContent200Response,
};
use async_trait::async_trait;
use lgn_auth::UserInfo;
use lgn_online::server::{Error, Result};
use lgn_tracing::{async_span_scope, info};

use crate::{
    AliasProvider, ContentAddressProvider, ContentProvider, ContentReaderExt, ContentWriterExt,
    DataSpace, HashRef, Identifier,
};

pub struct ApiProviderSet {
    pub alias_provider: Box<dyn AliasProvider>,
    pub content_provider: Box<dyn ContentProvider>,
    pub content_address_provider: Box<dyn ContentAddressProvider>,
    pub size_threshold: usize,
}

#[derive(Default)]
pub struct Server {
    providers: HashMap<DataSpace, ApiProviderSet>,
}

impl Server {
    /// Instantiate a new `ApiImpl` with the given `Provider` and
    /// `AddressProvider`.
    ///
    /// Read and write requests are routed to the `Provider` if the size is
    /// below or equal the specified `size_threshold`.
    ///
    /// Otherwise, the request is routed to the `AddressProvider` to get the
    /// address of the downloader/uploader.
    pub fn new(providers: HashMap<DataSpace, ApiProviderSet>) -> Self {
        Self { providers }
    }
}

#[async_trait]
impl Api for Arc<Server> {
    async fn resolve_alias(&self, request: ResolveAliasRequest) -> Result<ResolveAliasResponse> {
        async_span_scope!("Server::resolve_alias");

        let user_info = request.parts.extensions.get::<UserInfo>().cloned();

        let data_space = request
            .data_space
            .try_into()
            .map_err(|err| Error::bad_request(format!("failed to parse data space: {}", err)))?;
        let key: Vec<u8> = request.alias_key.0.into();

        if let Some(user_info) = user_info {
            info!(
                "Received resolve_alias request for {}/{:02x?} from user {}",
                data_space,
                key,
                user_info.username()
            );
        }

        let provider_set = self.providers.get(&data_space).ok_or_else(|| {
            Error::internal(format!("no provider set for data space `{}`", data_space))
        })?;

        Ok(ResolveAliasResponse::Status200(ResolveAlias200Response {
            id: match provider_set.alias_provider.resolve_alias(&key).await {
                Ok(id) => id.to_string().into(),
                Err(crate::alias_providers::Error::AliasNotFound { .. }) => "".to_string().into(),
                Err(err) => {
                    return Err(Error::internal(format!("failed to resolve alias: {}", err)))
                }
            },
        }))
    }

    async fn register_alias(&self, request: RegisterAliasRequest) -> Result<RegisterAliasResponse> {
        async_span_scope!("Server::register_alias");

        let user_info = request.parts.extensions.get::<UserInfo>().cloned();

        let data_space = request
            .data_space
            .try_into()
            .map_err(|err| Error::bad_request(format!("failed to parse data space: {}", err)))?;
        let key: Vec<u8> = request.alias_key.0.into();
        let id: HashRef = request
            .content_id
            .try_into()
            .map_err(|err| Error::bad_request(format!("failed to parse identifier: {}", err)))?;

        if let Some(user_info) = user_info {
            info!(
                "Received register_alias request for {:02x?} as {} from user {}",
                key,
                id,
                user_info.username()
            );
        }

        let provider_set = self.providers.get(&data_space).ok_or_else(|| {
            Error::internal(format!("no provider set for data space `{}`", data_space))
        })?;

        match provider_set
            .alias_provider
            .register_alias(&key, &Identifier::new_hash_ref(id))
            .await
        {
            Ok(id) => Ok(RegisterAliasResponse::Status201(RegisterAlias201Response {
                id: id.to_string().into(),
            })),
            Err(crate::alias_providers::Error::AliasAlreadyExists(_)) => {
                Ok(RegisterAliasResponse::Status409)
            }
            Err(err) => {
                return Err(Error::internal(format!(
                    "failed to register alias: {}",
                    err
                )))
            }
        }
    }

    async fn read_content(&self, request: ReadContentRequest) -> Result<ReadContentResponse> {
        async_span_scope!("Server::read_content");

        let user_info = request.parts.extensions.get::<UserInfo>().cloned();

        let data_space = request
            .data_space
            .try_into()
            .map_err(|err| Error::bad_request(format!("failed to parse data space: {}", err)))?;
        let id: HashRef = request
            .content_id
            .try_into()
            .map_err(|err| Error::bad_request(format!("failed to parse identifier: {}", err)))?;

        if let Some(user_info) = user_info {
            info!(
                "Received read_content request for {} from user {}",
                id,
                user_info.username()
            );
        }

        let provider_set = self.providers.get(&data_space).ok_or_else(|| {
            Error::internal(format!("no provider set for data space `{}`", data_space))
        })?;

        if id.data_size() <= provider_set.size_threshold {
            match provider_set
                .content_provider
                .read_content_with_origin(&id)
                .await
            {
                Ok((data, origin)) => Ok(ReadContentResponse::Status200 {
                    body: data.into(),
                    x_origin: Origin(rmp_serde::to_vec(&origin).unwrap().into()),
                }),
                Err(crate::content_providers::Error::HashRefNotFound(_)) => {
                    Ok(ReadContentResponse::Status404)
                }
                Err(err) => Err(Error::internal(format!("failed to read content: {}", err))),
            }
        } else {
            match provider_set
                .content_address_provider
                .get_content_read_address_with_origin(&id)
                .await
            {
                Ok((url, origin)) => Ok(ReadContentResponse::Status204 {
                    x_url: url.into(),
                    x_origin: Origin(rmp_serde::to_vec(&origin).unwrap().into()),
                }),
                Err(crate::content_providers::Error::HashRefNotFound(_)) => {
                    Ok(ReadContentResponse::Status404)
                }
                Err(err) => Err(Error::internal(format!(
                    "failed to read content address: {}",
                    err
                ))),
            }
        }
    }

    async fn write_content(&self, request: WriteContentRequest) -> Result<WriteContentResponse> {
        async_span_scope!("Server::write_content");

        let user_info = request.parts.extensions.get::<UserInfo>().cloned();
        let data_space = request
            .data_space
            .try_into()
            .map_err(|err| Error::bad_request(format!("failed to parse data space: {}", err)))?;

        if let Some(user_info) = user_info {
            info!(
                "Received write_content request from user {}",
                user_info.username()
            );
        }

        let provider_set = self.providers.get(&data_space).ok_or_else(|| {
            Error::internal(format!("no provider set for data space `{}`", data_space))
        })?;

        let data = request.body;

        if data.len() > provider_set.size_threshold as usize {
            return Err(Error::internal(format!(
                "refusing to write content of size {} that exceeds the size threshold of {}",
                data.len(),
                provider_set.size_threshold
            )));
        }

        let id = provider_set
            .content_provider
            .write_content(&data)
            .await
            .map_err(|err| Error::internal(format!("failed to write content: {}", err)))?;

        Ok(WriteContentResponse::Status200(WriteContent200Response {
            id: ContentId(id.to_string()),
        }))
    }

    async fn get_content_writer(
        &self,
        request: GetContentWriterRequest,
    ) -> Result<GetContentWriterResponse> {
        async_span_scope!("Server::get_content_writer");

        let user_info = request.parts.extensions.get::<UserInfo>().cloned();
        let data_space = request
            .data_space
            .try_into()
            .map_err(|err| Error::bad_request(format!("failed to parse data space: {}", err)))?;
        let id: HashRef = request
            .content_id
            .try_into()
            .map_err(|err| Error::bad_request(format!("failed to parse identifier: {}", err)))?;

        if let Some(user_info) = user_info {
            info!(
                "Received get_content_writer request for {} from user {}",
                id,
                user_info.username()
            );
        }

        let provider_set = self.providers.get(&data_space).ok_or_else(|| {
            Error::internal(format!("no provider set for data space `{}`", data_space))
        })?;

        if id.data_size() <= provider_set.size_threshold {
            match provider_set.content_provider.get_content_writer(&id).await {
                Ok(_) => {
                    Ok(GetContentWriterResponse::Status200(
                        GetContentWriter200Response {
                            // An empty URL means that the content is small enough to be
                            // fetched directly from the provider and passed through the
                            // gRPC stream.
                            url: Url("".to_string()),
                        },
                    ))
                }
                Err(crate::content_providers::Error::HashRefAlreadyExists(_)) => {
                    Ok(GetContentWriterResponse::Status409)
                }
                Err(err) => {
                    return Err(Error::internal(format!(
                        "failed to read content address: {}",
                        err
                    )))
                }
            }
        } else {
            match provider_set
                .content_address_provider
                .get_content_write_address(&id)
                .await
            {
                Ok(url) => Ok(GetContentWriterResponse::Status200(
                    GetContentWriter200Response { url: Url(url) },
                )),
                Err(crate::content_providers::Error::HashRefAlreadyExists(_)) => {
                    Ok(GetContentWriterResponse::Status409)
                }
                Err(err) => {
                    return Err(Error::internal(format!(
                        "failed to read content address: {}",
                        err
                    )))
                }
            }
        }
    }
}
