use std::collections::HashMap;

use async_trait::async_trait;
use lgn_content_store_proto::{
    read_content_response::Content, DataContent, GetContentWriterRequest, GetContentWriterResponse,
    ReadContentRequest, ReadContentResponse, RegisterAliasRequest, RegisterAliasResponse,
    ResolveAliasRequest, ResolveAliasResponse, UrlContent, WriteContentRequest,
    WriteContentResponse,
};
use lgn_online::authentication::UserInfo;
use lgn_tracing::{async_span_scope, info};
use tonic::{Request, Response};

use crate::{
    AliasProvider, ContentAddressProvider, ContentProvider, ContentReaderExt, ContentWriterExt,
    DataSpace, HashRef, Identifier, Result,
};

pub struct GrpcProviderSet {
    pub alias_provider: Box<dyn AliasProvider>,
    pub content_provider: Box<dyn ContentProvider>,
    pub content_address_provider: Box<dyn ContentAddressProvider>,
    pub size_threshold: usize,
}

pub struct GrpcService {
    providers: HashMap<DataSpace, GrpcProviderSet>,
}

impl GrpcService {
    /// Instantiate a new `GrpcService` with the given `Provider` and
    /// `AddressProvider`.
    ///
    /// Read and write requests are routed to the `Provider` if the size is
    /// below or equal the specified `size_threshold`.
    ///
    /// Otherwise, the request is routed to the `AddressProvider` to get the
    /// address of the downloader/uploader.
    pub fn new(providers: HashMap<DataSpace, GrpcProviderSet>) -> Self {
        Self { providers }
    }
}

#[async_trait]
impl lgn_content_store_proto::content_store_server::ContentStore for GrpcService {
    async fn resolve_alias(
        &self,
        request: Request<ResolveAliasRequest>,
    ) -> Result<Response<ResolveAliasResponse>, tonic::Status> {
        async_span_scope!("GrpcServer::resolve_alias");

        let user_info = request.extensions().get::<UserInfo>().cloned();

        let request = request.into_inner();

        let data_space = request.data_space.parse().map_err(|err| {
            tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("failed to parse data space: {}", err),
            )
        })?;
        let key = request.key;

        if let Some(user_info) = user_info {
            info!(
                "Received resolve_alias request for {}/{:02x?} from user {}",
                data_space,
                key,
                user_info.username()
            );
        }

        let provider_set = self.providers.get(&data_space).ok_or_else(|| {
            tonic::Status::new(
                tonic::Code::Internal,
                format!("no provider set for data space `{}`", data_space),
            )
        })?;

        Ok(Response::new(ResolveAliasResponse {
            id: match provider_set.alias_provider.resolve_alias(&key).await {
                Ok(id) => id.to_string(),
                Err(crate::alias_providers::Error::AliasNotFound { .. }) => "".to_string(),
                Err(err) => {
                    return Err(tonic::Status::new(
                        tonic::Code::Internal,
                        format!("failed to resolve alias: {}", err),
                    ))
                }
            },
        }))
    }

    async fn register_alias(
        &self,
        request: Request<RegisterAliasRequest>,
    ) -> Result<Response<RegisterAliasResponse>, tonic::Status> {
        async_span_scope!("GrpcServer::register_alias");

        let user_info = request.extensions().get::<UserInfo>().cloned();

        let request = request.into_inner();

        let data_space = request.data_space.parse().map_err(|err| {
            tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("failed to parse data space: {}", err),
            )
        })?;
        let key = request.key;
        let id: HashRef = request.id.parse().map_err(|err| {
            tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("failed to parse identifier: {}", err),
            )
        })?;

        if let Some(user_info) = user_info {
            info!(
                "Received register_alias request for {:02x?} as {} from user {}",
                key,
                id,
                user_info.username()
            );
        }

        let provider_set = self.providers.get(&data_space).ok_or_else(|| {
            tonic::Status::new(
                tonic::Code::Internal,
                format!("no provider set for data space `{}`", data_space),
            )
        })?;

        Ok(Response::new(
            match provider_set
                .alias_provider
                .register_alias(&key, &Identifier::new_hash_ref(id))
                .await
            {
                Ok(id) => RegisterAliasResponse {
                    newly_registered: true,
                    id: id.to_string(),
                },
                Err(crate::alias_providers::Error::AliasAlreadyExists(_)) => {
                    RegisterAliasResponse {
                        newly_registered: false,
                        id: "".to_string(),
                    }
                }
                Err(err) => {
                    return Err(tonic::Status::new(
                        tonic::Code::Internal,
                        format!("failed to register alias: {}", err),
                    ))
                }
            },
        ))
    }

    async fn read_content(
        &self,
        request: Request<ReadContentRequest>,
    ) -> Result<Response<ReadContentResponse>, tonic::Status> {
        async_span_scope!("GrpcServer::read_content");

        let user_info = request.extensions().get::<UserInfo>().cloned();

        let request = request.into_inner();

        let data_space = request.data_space.parse().map_err(|err| {
            tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("failed to parse data space: {}", err),
            )
        })?;
        let id: HashRef = request.id.parse().map_err(|err| {
            tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("failed to parse identifier: {}", err),
            )
        })?;

        if let Some(user_info) = user_info {
            info!(
                "Received read_content request for {} from user {}",
                id,
                user_info.username()
            );
        }

        let provider_set = self.providers.get(&data_space).ok_or_else(|| {
            tonic::Status::new(
                tonic::Code::Internal,
                format!("no provider set for data space `{}`", data_space),
            )
        })?;

        let content = if id.data_size() <= provider_set.size_threshold {
            match provider_set
                .content_provider
                .read_content_with_origin(&id)
                .await
            {
                Ok((data, origin)) => Some(Content::Data(DataContent {
                    data,
                    origin: rmp_serde::to_vec(&origin).unwrap(),
                })),
                Err(crate::content_providers::Error::HashRefNotFound(_)) => None,
                Err(err) => {
                    return Err(tonic::Status::new(
                        tonic::Code::Internal,
                        format!("failed to read content: {}", err),
                    ))
                }
            }
        } else {
            match provider_set
                .content_address_provider
                .get_content_read_address_with_origin(&id)
                .await
            {
                Ok((url, origin)) => Some(Content::Url(UrlContent {
                    url,
                    origin: rmp_serde::to_vec(&origin).unwrap(),
                })),
                Err(crate::content_providers::Error::HashRefNotFound(_)) => None,
                Err(err) => {
                    return Err(tonic::Status::new(
                        tonic::Code::Internal,
                        format!("failed to read content address: {}", err),
                    ))
                }
            }
        };

        Ok(Response::new(ReadContentResponse { content }))
    }

    async fn get_content_writer(
        &self,
        request: Request<GetContentWriterRequest>,
    ) -> Result<Response<GetContentWriterResponse>, tonic::Status> {
        async_span_scope!("GrpcServer::get_content_writer");

        let user_info = request.extensions().get::<UserInfo>().cloned();

        let request = request.into_inner();
        let data_space = request.data_space.parse().map_err(|err| {
            tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("failed to parse data space: {}", err),
            )
        })?;
        let id: HashRef = request.id.parse().map_err(|err| {
            tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("failed to parse identifier: {}", err),
            )
        })?;

        if let Some(user_info) = user_info {
            info!(
                "Received get_content_writer request for {} from user {}",
                id,
                user_info.username()
            );
        }

        let provider_set = self.providers.get(&data_space).ok_or_else(|| {
            tonic::Status::new(
                tonic::Code::Internal,
                format!("no provider set for data space `{}`", data_space),
            )
        })?;

        Ok(Response::new(GetContentWriterResponse {
            content_writer: if id.data_size() <= provider_set.size_threshold {
                match provider_set.content_provider.get_content_writer(&id).await {
                    Ok(_) => {
                        // An empty URL means that the content is small enough to be
                        // fetched directly from the provider and passed through the
                        // gRPC stream.
                        Some(
                            lgn_content_store_proto::get_content_writer_response::ContentWriter::Url(
                                "".to_string(),
                            ),
                        )
                    }
                    Err(crate::content_providers::Error::HashRefAlreadyExists(_)) => None,
                    Err(err) => {
                        return Err(tonic::Status::new(
                            tonic::Code::Internal,
                            format!("failed to read content address: {}", err),
                        ))
                    }
                }
            } else {
                match provider_set
                    .content_address_provider
                    .get_content_write_address(&id)
                    .await
                {
                    Ok(url) => Some(
                        lgn_content_store_proto::get_content_writer_response::ContentWriter::Url(
                            url,
                        ),
                    ),
                    Err(crate::content_providers::Error::HashRefAlreadyExists(_)) => None,
                    Err(err) => {
                        return Err(tonic::Status::new(
                            tonic::Code::Internal,
                            format!("failed to read content address: {}", err),
                        ))
                    }
                }
            },
        }))
    }

    async fn write_content(
        &self,
        request: Request<WriteContentRequest>,
    ) -> Result<Response<WriteContentResponse>, tonic::Status> {
        async_span_scope!("GrpcServer::write_content");

        let user_info = request.extensions().get::<UserInfo>().cloned();

        let request = request.into_inner();
        let data_space = request.data_space.parse().map_err(|err| {
            tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("failed to parse data space: {}", err),
            )
        })?;

        if let Some(user_info) = user_info {
            info!(
                "Received write_content request from user {}",
                user_info.username()
            );
        }

        let provider_set = self.providers.get(&data_space).ok_or_else(|| {
            tonic::Status::new(
                tonic::Code::Internal,
                format!("no provider set for data space `{}`", data_space),
            )
        })?;

        let data = request.data;

        if data.len() > provider_set.size_threshold as usize {
            return Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!(
                    "refusing to write content of size {} that exceeds the size threshold of {}",
                    data.len(),
                    provider_set.size_threshold
                ),
            ));
        }

        let id = provider_set
            .content_provider
            .write_content(&data)
            .await
            .map_err(|err| {
                tonic::Status::new(
                    tonic::Code::Internal,
                    format!("failed to write content: {}", err),
                )
            })?;

        Ok(Response::new(WriteContentResponse { id: id.to_string() }))
    }
}
