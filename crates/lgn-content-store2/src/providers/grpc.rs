use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_trait::async_trait;
use futures::{stream::TryStreamExt, Future};
use http_body::Body;
use lgn_content_store_proto::{
    content_store_client::ContentStoreClient, read_content_response::Content,
    GetContentWriterRequest, GetContentWriterResponse, ReadContentRequest, ReadContentResponse,
    WriteContentRequest, WriteContentResponse,
};
use pin_project::pin_project;
use tokio::{io::AsyncWrite, sync::Mutex};
use tokio_util::{compat::FuturesAsyncReadCompatExt, io::ReaderStream};
use tonic::{codegen::StdError, Request, Response};

use crate::{
    traits::ContentAddressProvider, ContentAsyncRead, ContentAsyncWrite, ContentProvider,
    ContentReader, ContentWriter, Error, Identifier, Result,
};

/// A `GrpcProvider` is a provider that delegates to a `gRPC` service.
pub struct GrpcProvider<C> {
    client: Arc<Mutex<ContentStoreClient<C>>>,
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

        Self { client }
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
                        Arc::clone(&self.client),
                        id.clone(),
                    )))
                } else {
                    let uploader = HttpUploader::new(url)?;

                    Ok(Box::pin(uploader))
                }
            }
            None => Err(Error::AlreadyExists),
        }
    }
}

#[pin_project]
struct GrpcUploader<C> {
    #[pin]
    state: GrpcUploaderState<C>,
}

#[allow(clippy::type_complexity)]
enum GrpcUploaderState<C> {
    Writing(
        Option<(
            std::io::Cursor<Vec<u8>>,
            Identifier,
            Arc<Mutex<ContentStoreClient<C>>>,
        )>,
    ),
    Uploading(Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'static>>),
}

impl<C> GrpcUploader<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send + 'static,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    pub fn new(client: Arc<Mutex<ContentStoreClient<C>>>, id: Identifier) -> Self {
        let state =
            GrpcUploaderState::Writing(Some((std::io::Cursor::new(Vec::new()), id, client)));

        Self { state }
    }

    async fn upload(
        data: Vec<u8>,
        id: Identifier,
        client: Arc<Mutex<ContentStoreClient<C>>>,
    ) -> Result<(), std::io::Error> {
        id.matches(&data).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                anyhow::anyhow!("the data does not match the specified id: {}", err),
            )
        })?;

        let req = lgn_content_store_proto::WriteContentRequest { data };

        let res_id: Identifier = client
            .lock()
            .await
            .write_content(req)
            .await
            .map_err(|err| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    anyhow::anyhow!("gRPC request failed: {}", err),
                )
            })?
            .into_inner()
            .id
            .parse()
            .map_err(|err| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    anyhow::anyhow!("failed to parse response id: {}", err),
                )
            })?;

        if res_id != id {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                anyhow::anyhow!("the response id does not match the request id"),
            ))
        } else {
            Ok(())
        }
    }
}

impl<C> AsyncWrite for GrpcUploader<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send + 'static,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        let this = self.project();

        if let GrpcUploaderState::Writing(Some((cursor, _, _))) = this.state.get_mut() {
            Pin::new(cursor).poll_write(cx, buf)
        } else {
            panic!("HttpUploader::poll_write called after completion")
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let this = self.project();

        if let GrpcUploaderState::Writing(Some((cursor, _, _))) = this.state.get_mut() {
            Pin::new(cursor).poll_flush(cx)
        } else {
            panic!("HttpUploader::poll_flush called after completion")
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let this = self.project();
        let state = this.state.get_mut();

        loop {
            *state = match state {
                GrpcUploaderState::Writing(args) => {
                    let res = Pin::new(&mut args.as_mut().unwrap().0).poll_shutdown(cx);

                    match res {
                        Poll::Ready(Ok(())) => {
                            let (cursor, id, client) = args.take().unwrap();

                            GrpcUploaderState::Uploading(Box::pin(Self::upload(
                                cursor.into_inner(),
                                id,
                                client,
                            )))
                        }
                        p => return p,
                    }
                }
                GrpcUploaderState::Uploading(call) => return Pin::new(call).poll(cx),
            };
        }
    }
}

#[pin_project]
struct HttpUploader {
    #[pin]
    w: Option<tokio_pipe::PipeWrite>,

    #[pin]
    call: Pin<Box<dyn Future<Output = Result<reqwest::Response, reqwest::Error>> + Send + 'static>>,
}

impl HttpUploader {
    pub fn new(url: String) -> Result<Self> {
        let client = reqwest::Client::new();

        let (r, w) =
            tokio_pipe::pipe().map_err(|err| anyhow::anyhow!("failed to create pipe: {}", err))?;

        let stream = ReaderStream::new(r);
        let w = Some(w);
        let call = Box::pin(
            client
                .put(url)
                .body(reqwest::Body::wrap_stream(stream))
                .send(),
        );

        Ok(Self { w, call })
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
    size_threshold: u64,
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
    pub fn new(provider: Provider, address_provider: AddressProvider, size_threshold: u64) -> Self {
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
        let id: Identifier = request.into_inner().id.parse().map_err(|err| {
            tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("failed to parse identifier: {}", err),
            )
        })?;

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
        let id: Identifier = request.into_inner().id.parse().map_err(|err| {
            tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("failed to parse identifier: {}", err),
            )
        })?;

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
