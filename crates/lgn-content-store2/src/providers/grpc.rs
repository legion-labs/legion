use std::{
    collections::{BTreeMap, BTreeSet},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_trait::async_trait;
use futures::{stream::TryStreamExt, Future};
use http::header;
use http_body::Body;
use lgn_content_store_proto::{
    content_store_client::ContentStoreClient, read_content_response::Content,
    GetContentWriterRequest, GetContentWriterResponse, ReadContentRequest, ReadContentResponse,
    WriteContentRequest, WriteContentResponse,
};
use lgn_online::authentication::UserInfo;
use lgn_tracing::{debug, info};
use pin_project::pin_project;
use tokio::{io::AsyncWrite, sync::Mutex};
use tokio_util::{compat::FuturesAsyncReadCompatExt, io::ReaderStream};
use tonic::{codegen::StdError, Request, Response};

use crate::{
    traits::{
        get_content_readers_impl, ContentAddressProvider, ContentReaderExt, ContentWriterExt,
    },
    ContentAsyncRead, ContentAsyncWrite, ContentProvider, ContentReader, ContentWriter, Error,
    Identifier, Result,
};

use super::{Uploader, UploaderImpl};

/// A `GrpcProvider` is a provider that delegates to a `gRPC` service.
#[derive(Debug, Clone)]
pub struct GrpcProvider<C> {
    client: Arc<Mutex<ContentStoreClient<C>>>,
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
    pub async fn new(grpc_client: C) -> Self {
        let client = Arc::new(Mutex::new(ContentStoreClient::new(grpc_client)));
        // The buffer for HTTP uploaders is set to 2MB.
        let buf_size = 2 * 1024 * 1024;

        Self { client, buf_size }
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
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        let req = lgn_content_store_proto::ReadContentRequest { id: id.to_string() };

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
                Content::Data(data) => Box::pin(std::io::Cursor::new(data)),
                Content::Url(url) => Box::pin(
                    match reqwest::get(&url)
                        .await
                        .map_err(|err| {
                            anyhow::anyhow!("failed to fetch content from {}: {}", url, err)
                        })?
                        .error_for_status()
                    {
                        Ok(resp) => resp
                            .bytes_stream()
                            .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
                            .into_async_read()
                            .compat(),
                        Err(err) => {
                            return match err.status() {
                                Some(reqwest::StatusCode::NOT_FOUND) => Err(Error::NotFound),
                                _ => Err(anyhow::anyhow!("HTTP error: {}", err).into()),
                            }
                        }
                    },
                ),
            }),
            None => Err(Error::NotFound),
        }
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncRead>>> {
        get_content_readers_impl(self, ids).await
    }
}

#[async_trait]
impl<C> ContentWriter for GrpcProvider<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send + 'static,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        let req = lgn_content_store_proto::GetContentWriterRequest { id: id.to_string() };

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
                        },
                    )))
                } else {
                    let uploader = HttpUploader::new(id.clone(), url, self.buf_size);

                    Ok(Box::pin(uploader))
                }
            }
            None => Err(Error::AlreadyExists),
        }
    }
}

type GrpcUploader<C> = Uploader<GrpcUploaderImpl<C>>;

struct GrpcUploaderImpl<C> {
    client: Arc<Mutex<ContentStoreClient<C>>>,
}

#[async_trait]
impl<C> UploaderImpl for GrpcUploaderImpl<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send + 'static,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    async fn upload(self, data: Vec<u8>, id: Identifier) -> Result<()> {
        let req = lgn_content_store_proto::WriteContentRequest { data };

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

pub struct GrpcService<Provider, AddressProvider> {
    provider: Provider,
    address_provider: AddressProvider,
    size_threshold: usize,
}

impl<Provider, AddressProvider> GrpcService<Provider, AddressProvider> {
    /// Instanciate a new `GrpcService` with the given `Provider` and
    /// `AddressProvider`.
    ///
    /// Read and write requests are routed to the `Provider` if the size is
    /// below or equal the specified `size_threshold`.
    ///
    /// Otherwise, the request is routed to the `AddressProvider` to get the
    /// address of the downloader/uploader.
    pub fn new(
        provider: Provider,
        address_provider: AddressProvider,
        size_threshold: usize,
    ) -> Self {
        Self {
            provider,
            address_provider,
            size_threshold,
        }
    }
}

#[async_trait]
impl<Provider, AddressProvider> lgn_content_store_proto::content_store_server::ContentStore
    for GrpcService<Provider, AddressProvider>
where
    Provider: ContentProvider + Send + Sync + 'static,
    AddressProvider: ContentAddressProvider + Send + Sync + 'static,
{
    async fn read_content(
        &self,
        request: Request<ReadContentRequest>,
    ) -> Result<Response<ReadContentResponse>, tonic::Status> {
        let user_info = request.extensions().get::<UserInfo>().cloned();

        let id: Identifier = request.into_inner().id.parse().map_err(|err| {
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

        Ok(Response::new(ReadContentResponse {
            content: if id.data_size() <= self.size_threshold {
                match self.provider.read_content(&id).await {
                    Ok(data) => Some(Content::Data(data)),
                    Err(Error::NotFound) => None,
                    Err(err) => {
                        return Err(tonic::Status::new(
                            tonic::Code::Internal,
                            format!("failed to read content: {}", err),
                        ))
                    }
                }
            } else {
                match self.address_provider.get_content_read_address(&id).await {
                    Ok(url) => Some(Content::Url(url)),
                    Err(Error::NotFound) => None,
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

    async fn get_content_writer(
        &self,
        request: Request<GetContentWriterRequest>,
    ) -> Result<Response<GetContentWriterResponse>, tonic::Status> {
        let user_info = request.extensions().get::<UserInfo>().cloned();

        let id: Identifier = request.into_inner().id.parse().map_err(|err| {
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

        Ok(Response::new(GetContentWriterResponse {
            content_writer: if id.data_size() <= self.size_threshold {
                // An empty URL means that the content is small enough to be
                // fetched directly from the provider and passed through the
                // gRPC stream.
                Some(
                    lgn_content_store_proto::get_content_writer_response::ContentWriter::Url(
                        "".to_string(),
                    ),
                )
            } else {
                match self.address_provider.get_content_write_address(&id).await {
                    Ok(url) => Some(
                        lgn_content_store_proto::get_content_writer_response::ContentWriter::Url(
                            url,
                        ),
                    ),
                    Err(Error::AlreadyExists) => None,
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
        let user_info = request.extensions().get::<UserInfo>().cloned();

        if let Some(user_info) = user_info {
            info!(
                "Received write_content request from user {}",
                user_info.username()
            );
        }

        let data = request.into_inner().data;

        if data.len() > self.size_threshold as usize {
            return Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!(
                    "refusing to write content of size {} that exceeds the size threshold of {}",
                    data.len(),
                    self.size_threshold
                ),
            ));
        }

        let id = self.provider.write_content(&data).await.map_err(|err| {
            tonic::Status::new(
                tonic::Code::Internal,
                format!("failed to write content: {}", err),
            )
        })?;

        Ok(Response::new(WriteContentResponse { id: id.to_string() }))
    }
}
