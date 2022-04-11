use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt::{Debug, Display},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_trait::async_trait;
use futures::{stream::TryStreamExt, Future};
use http::header;
use http_body::Body;
use lgn_content_store_proto::{
    content_store_client::ContentStoreClient, read_content_response::Content, DataContent,
    GetContentWriterRequest, GetContentWriterResponse, ReadContentRequest, ReadContentResponse,
    RegisterAliasRequest, RegisterAliasResponse, ResolveAliasRequest, ResolveAliasResponse,
    UrlContent, WriteContentRequest, WriteContentResponse,
};
use lgn_online::authentication::UserInfo;
use lgn_tracing::{async_span_scope, debug, error, info, warn};
use pin_project::pin_project;
use tokio::{io::AsyncWrite, sync::Mutex};
use tokio_util::{compat::FuturesAsyncReadCompatExt, io::ReaderStream};
use tonic::{codegen::StdError, Request, Response};

use crate::{
    traits::{
        get_content_readers_impl, ContentAddressProvider, ContentReaderExt, ContentWriterExt,
        WithOrigin,
    },
    ContentAsyncReadWithOrigin, ContentAsyncWrite, ContentProvider, ContentReader, ContentWriter,
    DataSpace, Error, Identifier, Result,
};

use super::{Uploader, UploaderImpl};

/// A `GrpcProvider` is a provider that delegates to a `gRPC` service.
#[derive(Debug, Clone)]
pub struct GrpcProvider<C> {
    client: Arc<Mutex<ContentStoreClient<C>>>,
    data_space: DataSpace,
    buf_size: usize,
}

impl<C> GrpcProvider<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    pub async fn new(grpc_client: C, data_space: DataSpace) -> Self {
        let client = Arc::new(Mutex::new(ContentStoreClient::new(grpc_client)));
        // The buffer for HTTP uploaders is set to 2MB.
        let buf_size = 2 * 1024 * 1024;

        Self {
            client,
            data_space,
            buf_size,
        }
    }
}

impl<C> Display for GrpcProvider<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "gRPC client (data space: {})", self.data_space)
    }
}

#[async_trait]
impl<C> ContentReader for GrpcProvider<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncReadWithOrigin> {
        async_span_scope!("GrpcProvider::get_content_reader");

        debug!("GrpcProvider::get_content_reader({})", id);

        let req = lgn_content_store_proto::ReadContentRequest {
            data_space: self.data_space.to_string(),
            id: id.to_string(),
        };

        let resp = self
            .client
            .lock()
            .await
            .read_content(req)
            .await
            .map_err(|err| anyhow::anyhow!("gRPC request failed: {}", err))?
            .into_inner();

        match resp.content {
            Some(content) => Ok(match content {
                Content::Data(content) => {
                    debug!(
                        "GrpcProvider::get_content_reader({}) -> content data is available",
                        id
                    );

                    let origin = rmp_serde::from_slice(&content.origin)
                        .map_err(|err| anyhow::anyhow!("failed to parse origin: {}", err))?;

                    std::io::Cursor::new(content.data).with_origin(origin)
                }
                Content::Url(content) => {
                    debug!(
                        "GrpcProvider::get_content_reader({}) -> content URL is available",
                        id
                    );

                    let origin = rmp_serde::from_slice(&content.origin)
                        .map_err(|err| anyhow::anyhow!("failed to parse origin: {}", err))?;

                    match reqwest::get(&content.url)
                        .await
                        .map_err(|err| {
                            anyhow::anyhow!("failed to fetch content from {}: {}", content.url, err)
                        })?
                        .error_for_status()
                    {
                        Ok(resp) => {
                            debug!(
                                "GrpcProvider::get_content_reader({}) -> started reading content from URL...",
                                id
                            );

                             resp
                            .bytes_stream()
                            .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
                            .into_async_read()
                            .compat()
                        }
                        Err(err) => {
                            return if err.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                                warn!(
                                    "GrpcProvider::get_content_reader({}) -> content does not exist at the specified URL.",
                                    id
                                );

                                Err(Error::IdentifierNotFound(id.clone()))
                            } else {
                                error!(
                                    "GrpcProvider::get_content_reader({}) -> failed to read content from the specified URL: {}",
                                    id, err
                                );

                                Err(anyhow::anyhow!("HTTP error: {}", err).into())
                            }
                        }
                    }
                    .with_origin(origin)
                }
            }),
            None => {
                warn!(
                    "GrpcProvider::get_content_reader({}) -> content does not exist",
                    id
                );

                Err(Error::IdentifierNotFound(id.clone()))
            }
        }
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncReadWithOrigin>>> {
        async_span_scope!("GrpcProvider::get_content_readers");

        debug!("GrpcProvider::get_content_readers({:?})", ids);

        get_content_readers_impl(self, ids).await
    }

    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        async_span_scope!("GrpcProvider::resolve_alias");

        let req = lgn_content_store_proto::ResolveAliasRequest {
            data_space: self.data_space.to_string(),
            key_space: key_space.to_string(),
            key: key.to_string(),
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
            Err(Error::AliasNotFound {
                key_space: key_space.to_string(),
                key: key.to_string(),
            })
        } else {
            resp.id.parse()
        }
    }
}

#[async_trait]
impl<C> ContentWriter for GrpcProvider<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send + Debug + 'static,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        async_span_scope!("GrpcProvider::get_content_writer");

        let req = lgn_content_store_proto::GetContentWriterRequest {
            data_space: self.data_space.to_string(),
            id: id.to_string(),
        };

        let resp = self
            .client
            .lock()
            .await
            .get_content_writer(req)
            .await
            .map_err(|err| anyhow::anyhow!("gRPC request failed: {}", err))?
            .into_inner();

        match resp.content_writer {
            Some(lgn_content_store_proto::get_content_writer_response::ContentWriter::Url(url)) => {
                if url.is_empty() {
                    Ok(Box::pin(GrpcUploader::new(
                        id.clone(),
                        GrpcUploaderImpl {
                            client: Arc::clone(&self.client),
                            data_space: self.data_space.clone(),
                        },
                    )))
                } else {
                    let uploader = HttpUploader::new(id.clone(), url, self.buf_size);

                    Ok(Box::pin(uploader))
                }
            }
            None => Err(Error::IdentifierAlreadyExists(id.clone())),
        }
    }

    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        async_span_scope!("GrpcProvider::register_alias");

        let req = lgn_content_store_proto::RegisterAliasRequest {
            data_space: self.data_space.to_string(),
            key_space: key_space.to_string(),
            key: key.to_string(),
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
            Ok(())
        } else {
            Err(Error::AliasAlreadyExists {
                key_space: key_space.to_string(),
                key: key.to_string(),
            })
        }
    }
}

type GrpcUploader<C> = Uploader<GrpcUploaderImpl<C>>;

#[derive(Debug)]
struct GrpcUploaderImpl<C> {
    client: Arc<Mutex<ContentStoreClient<C>>>,
    data_space: DataSpace,
}

#[async_trait]
impl<C> UploaderImpl for GrpcUploaderImpl<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send + Debug + 'static,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    async fn upload(self, data: Vec<u8>, id: Identifier) -> Result<()> {
        async_span_scope!("GrpcProvider::upload");

        let req = lgn_content_store_proto::WriteContentRequest {
            data_space: self.data_space.to_string(),
            data,
        };

        let res_id: Identifier = self
            .client
            .lock()
            .await
            .write_content(req)
            .await
            .map_err(|err| anyhow::anyhow!("gRPC request failed: {}", err))?
            .into_inner()
            .id
            .parse()
            .map_err(|err| anyhow::anyhow!("failed to parse response id: {}", err))?;

        if res_id != id {
            Err(anyhow::anyhow!("the response id does not match the request id").into())
        } else {
            Ok(())
        }
    }
}

#[pin_project]
struct HttpUploader {
    #[pin]
    w: Option<tokio::io::DuplexStream>,

    #[pin]
    call: Pin<Box<dyn Future<Output = Result<reqwest::Response, reqwest::Error>> + Send + 'static>>,

    id: Identifier,
}

impl HttpUploader {
    pub fn new(id: Identifier, url: String, buf_size: usize) -> Self {
        let client = reqwest::Client::new();

        let (r, w) = tokio::io::duplex(buf_size);

        let stream = ReaderStream::new(r);
        let w = Some(w);

        debug!(
            "Starting HTTP upload for asset {} to {} ({} byte(s))",
            &id,
            url,
            id.data_size()
        );

        let call = Box::pin(
            client
                .put(url)
                .header(header::CONTENT_LENGTH, id.data_size())
                .body(reqwest::Body::wrap_stream(stream))
                .send(),
        );

        Self { w, call, id }
    }
}

#[async_trait]
impl AsyncWrite for HttpUploader {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        let this = self.project();

        // Before writing, we poll the call future to see if it's ready and
        // allow it to progress.
        //
        // If it is ready, we should not be writing and we must fail.
        if let Poll::Ready(resp) = this.call.poll(cx) {
            return Poll::Ready(match resp {
                Ok(resp) => Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    anyhow::anyhow!(
                        "HTTP request unexpectedly completed while uploading content: {}",
                        resp.status()
                    ),
                )),
                Err(err) => Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
            });
        }

        if let Some(w) = this.w.get_mut() {
            Pin::new(w).poll_write(cx, buf)
        } else {
            panic!("HttpUploader::poll_write called after completion")
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let this = self.project();

        // Before flushing, we poll the call future to see if it's ready and
        // allow it to progress.
        //
        // If it is ready, we should not be flushing and we must fail.
        if let Poll::Ready(resp) = this.call.poll(cx) {
            return Poll::Ready(match resp {
                Ok(resp) => Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    anyhow::anyhow!(
                        "HTTP request unexpectedly completed while uploading content: {}",
                        resp.status()
                    ),
                )),
                Err(err) => Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
            });
        }

        if let Some(w) = this.w.get_mut() {
            Pin::new(w).poll_flush(cx)
        } else {
            panic!("HttpUploader::poll_flush called after completion")
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let this = self.project();
        let w = this.w.get_mut();

        match {
            if let Some(w) = w {
                Pin::new(w).poll_shutdown(cx)
            } else {
                Poll::Ready(Ok(()))
            }
        } {
            Poll::Ready(Ok(())) => {
                // Shutdown went fine: let's make sure we never poll the writer again.
                w.take();

                debug!("Completed HTTP upload for asset {}", this.id);
            }
            Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
            Poll::Pending => return Poll::Pending,
        };

        match this.call.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(resp)) => {
                if resp.status().is_success() {
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        anyhow::anyhow!(
                            "HTTP request unexpectedly completed while uploading content: {}",
                            resp.status()
                        ),
                    )))
                }
            }
            Poll::Ready(Err(err)) => {
                Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, err)))
            }
        }
    }
}

pub struct GrpcProviderSet {
    pub provider: Box<dyn ContentProvider + Send + Sync>,
    pub address_provider: Box<dyn ContentAddressProvider + Send + Sync>,
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
        let key_space = request.key_space;
        let key = request.key;

        if let Some(user_info) = user_info {
            info!(
                "Received resolve_alias request for {}/{}/{} from user {}",
                data_space,
                key_space,
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
            id: match provider_set.provider.resolve_alias(&key_space, &key).await {
                Ok(id) => id.to_string(),
                Err(Error::AliasNotFound { .. }) => "".to_string(),
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
        let key_space = request.key_space;
        let key = request.key;
        let id: Identifier = request.id.parse().map_err(|err| {
            tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("failed to parse identifier: {}", err),
            )
        })?;

        if let Some(user_info) = user_info {
            info!(
                "Received register_alias request for {}/{} as {} from user {}",
                key_space,
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

        Ok(Response::new(RegisterAliasResponse {
            newly_registered: match provider_set
                .provider
                .register_alias(&key_space, &key, &id)
                .await
            {
                Ok(()) => true,
                Err(Error::IdentifierAlreadyExists(_)) => false,
                Err(err) => {
                    return Err(tonic::Status::new(
                        tonic::Code::Internal,
                        format!("failed to register alias: {}", err),
                    ))
                }
            },
        }))
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
        let id: Identifier = request.id.parse().map_err(|err| {
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
            match provider_set.provider.read_content_with_origin(&id).await {
                Ok((data, origin)) => Some(Content::Data(DataContent {
                    data,
                    origin: rmp_serde::to_vec(&origin).unwrap(),
                })),
                Err(Error::IdentifierNotFound(_)) => None,
                Err(err) => {
                    return Err(tonic::Status::new(
                        tonic::Code::Internal,
                        format!("failed to read content: {}", err),
                    ))
                }
            }
        } else {
            match provider_set
                .address_provider
                .get_content_read_address_with_origin(&id)
                .await
            {
                Ok((url, origin)) => Some(Content::Url(UrlContent {
                    url,
                    origin: rmp_serde::to_vec(&origin).unwrap(),
                })),
                Err(Error::IdentifierNotFound(_)) => None,
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
        let id: Identifier = request.id.parse().map_err(|err| {
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
                // An empty URL means that the content is small enough to be
                // fetched directly from the provider and passed through the
                // gRPC stream.
                Some(
                    lgn_content_store_proto::get_content_writer_response::ContentWriter::Url(
                        "".to_string(),
                    ),
                )
            } else {
                match provider_set
                    .address_provider
                    .get_content_write_address(&id)
                    .await
                {
                    Ok(url) => Some(
                        lgn_content_store_proto::get_content_writer_response::ContentWriter::Url(
                            url,
                        ),
                    ),
                    Err(Error::IdentifierAlreadyExists(_)) => None,
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
            .provider
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
