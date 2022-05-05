use std::{
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
    content_store_client::ContentStoreClient, read_content_response::Content,
};
use lgn_tracing::{async_span_scope, debug, error, warn};
use pin_project::pin_project;
use tokio::{io::AsyncWrite, sync::Mutex};
use tokio_util::{compat::FuturesAsyncReadCompatExt, io::ReaderStream};
use tonic::codegen::StdError;

use super::{
    ContentAsyncReadWithOriginAndSize, ContentAsyncWrite, ContentReader, ContentWriter, Error,
    HashRef, Result, Uploader, UploaderImpl, WithOriginAndSize,
};

use crate::DataSpace;

/// A `GrpcContentProvider` is a provider that delegates to a `gRPC` service.
#[derive(Debug, Clone)]
pub struct GrpcContentProvider<C> {
    client: Arc<Mutex<ContentStoreClient<C>>>,
    data_space: DataSpace,
    buf_size: usize,
}

impl<C> GrpcContentProvider<C>
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

impl<C> Display for GrpcContentProvider<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "gRPC client (data space: {})", self.data_space)
    }
}

#[async_trait]
impl<C> ContentReader for GrpcContentProvider<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send + Debug,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    async fn get_content_reader(&self, id: &HashRef) -> Result<ContentAsyncReadWithOriginAndSize> {
        async_span_scope!("GrpcContentProvider::get_content_reader");

        debug!("GrpcContentProvider::get_content_reader({})", id);

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
                        "GrpcContentProvider::get_content_reader({}) -> content data is available",
                        id
                    );

                    let origin = rmp_serde::from_slice(&content.origin)
                        .map_err(|err| anyhow::anyhow!("failed to parse origin: {}", err))?;

                    std::io::Cursor::new(content.data).with_origin_and_size(origin, id.data_size())
                }
                Content::Url(content) => {
                    debug!(
                        "GrpcContentProvider::get_content_reader({}) -> content URL is available",
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
                                "GrpcContentProvider::get_content_reader({}) -> started reading content from URL...",
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
                                    "GrpcContentProvider::get_content_reader({}) -> content does not exist at the specified URL.",
                                    id
                                );

                                Err(Error::HashRefNotFound(id.clone()))
                            } else {
                                error!(
                                    "GrpcContentProvider::get_content_reader({}) -> failed to read content from the specified URL: {}",
                                    id, err
                                );

                                Err(anyhow::anyhow!("HTTP error: {}", err).into())
                            }
                        }
                    }
                    .with_origin_and_size(origin, id.data_size())
                }
            }),
            None => {
                warn!(
                    "GrpcContentProvider::get_content_reader({}) -> content does not exist",
                    id
                );

                Err(Error::HashRefNotFound(id.clone()))
            }
        }
    }
}

#[async_trait]
impl<C> ContentWriter for GrpcContentProvider<C>
where
    C: tonic::client::GrpcService<tonic::body::BoxBody> + Send + Debug + 'static,
    C::ResponseBody: Body + Send + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    <C::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    async fn get_content_writer(&self, id: &HashRef) -> Result<ContentAsyncWrite> {
        async_span_scope!("GrpcContentProvider::get_content_writer");

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
                    Ok(Box::pin(GrpcUploader::new(GrpcUploaderImpl {
                        client: Arc::clone(&self.client),
                        data_space: self.data_space.clone(),
                    })))
                } else {
                    let uploader = HttpUploader::new(id.clone(), url, self.buf_size);

                    Ok(Box::pin(uploader))
                }
            }
            None => Err(Error::HashRefAlreadyExists(id.clone())),
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
    async fn upload(self, data: Vec<u8>) -> Result<()> {
        async_span_scope!("GrpcContentProvider::upload");

        let id = HashRef::new_from_data(&data);
        let req = lgn_content_store_proto::WriteContentRequest {
            data_space: self.data_space.to_string(),
            data,
        };

        let res_id: HashRef = self
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
            Err(Error::UnexpectedHashRef {
                expected: id,
                actual: res_id,
            })
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

    id: HashRef,
}

impl HttpUploader {
    pub fn new(id: HashRef, url: String, buf_size: usize) -> Self {
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

#[cfg(test)]
mod test {
    use futures::Stream;
    use hyper::server::{
        accept::Accept,
        conn::{AddrIncoming, AddrStream},
    };
    use lgn_online::grpc::GrpcClient;
    use std::net::SocketAddr;

    use crate::{
        ContentAddressReader, ContentAddressWriter, ContentReaderExt, ContentWriterExt,
        GrpcProviderSet, GrpcService, MemoryAliasProvider, MemoryContentProvider, Origin,
    };

    use super::*;

    #[derive(Debug)]
    pub struct FakeContentAddressProvider {
        base_url: String,
        already_exists: Arc<Mutex<bool>>,
    }

    impl Display for FakeContentAddressProvider {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "fake provider (base_url: {})", self.base_url)
        }
    }

    impl FakeContentAddressProvider {
        pub fn new(base_url: String) -> Self {
            Self {
                base_url,
                already_exists: Arc::new(Mutex::new(false)),
            }
        }

        pub fn get_address(&self, id: &HashRef, suffix: &str) -> String {
            format!("{}{}/{}", self.base_url, id, suffix)
        }

        pub async fn set_already_exists(&self, exists: bool) {
            *self.already_exists.lock().await = exists;
        }
    }

    #[async_trait]
    impl ContentAddressReader for FakeContentAddressProvider {
        async fn get_content_read_address_with_origin(
            &self,
            id: &HashRef,
        ) -> Result<(String, Origin)> {
            Ok((
                self.get_address(id, "read"),
                Origin::Local {
                    path: "fake".into(),
                },
            ))
        }
    }

    #[async_trait]
    impl ContentAddressWriter for FakeContentAddressProvider {
        async fn get_content_write_address(&self, id: &HashRef) -> Result<String> {
            if *self.already_exists.lock().await {
                Err(Error::HashRefAlreadyExists(id.clone()))
            } else {
                Ok(self.get_address(id, "write"))
            }
        }
    }

    pub struct TcpIncoming {
        inner: AddrIncoming,
    }

    impl TcpIncoming {
        pub(crate) fn new() -> Result<Self, anyhow::Error> {
            let mut inner = AddrIncoming::bind(&"127.0.0.1:0".parse()?)?;
            inner.set_nodelay(true);
            Ok(Self { inner })
        }

        pub(crate) fn addr(&self) -> SocketAddr {
            self.inner.local_addr()
        }
    }

    impl Stream for TcpIncoming {
        type Item = Result<AddrStream, std::io::Error>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Pin::new(&mut self.inner).poll_accept(cx)
        }
    }

    #[tokio::test]
    async fn test_grpc_content_provider() {
        // To debug this test more easily, you may want to specify: RUST_LOG=httptest=debug
        let _ = pretty_env_logger::try_init();

        let content_provider = MemoryContentProvider::new();
        let alias_provider = MemoryAliasProvider::new();

        let http_server = httptest::Server::run();

        const SMALL_DATA: [u8; 128] = [0x41; 128];
        const BIG_DATA: [u8; 512] = [0x41; 512];

        let address_provider = Arc::new(FakeContentAddressProvider::new(
            http_server.url("/").to_string(),
        ));
        let data_space = DataSpace::persistent();
        let providers = vec![(
            data_space.clone(),
            GrpcProviderSet {
                content_provider: Box::new(content_provider),
                alias_provider: Box::new(alias_provider),
                content_address_provider: Box::new(Arc::clone(&address_provider)),
                size_threshold: SMALL_DATA.len(),
            },
        )]
        .into_iter()
        .collect();

        let service = GrpcService::new(providers);
        let service =
            lgn_content_store_proto::content_store_server::ContentStoreServer::new(service);
        let server = tonic::transport::Server::builder().add_service(service);

        let incoming = TcpIncoming::new().unwrap();
        let addr = incoming.addr();

        async fn f(
            socket_addr: &SocketAddr,
            http_server: &httptest::Server,
            address_provider: Arc<FakeContentAddressProvider>,
            data_space: DataSpace,
        ) {
            let client = GrpcClient::new(format!("http://{}", socket_addr).parse().unwrap());
            let content_provider = GrpcContentProvider::new(client, data_space).await;

            let origin = Origin::Memory {};
            crate::content_providers::test_content_provider(&content_provider, &SMALL_DATA, origin)
                .await;

            // Now let's try again with a larger file.

            let id = HashRef::new_from_data(&BIG_DATA);

            http_server.expect(
                httptest::Expectation::matching(httptest::all_of![
                    httptest::matchers::request::method("GET"),
                    httptest::matchers::request::path(format!("/{}/read", id)),
                ])
                .respond_with(httptest::responders::status_code(404)),
            );

            match content_provider.get_content_reader(&id).await {
                Ok(_) => panic!("expected HashRefNotFound error"),
                Err(Error::HashRefNotFound(err_id)) => assert_eq!(err_id, id),
                Err(err) => panic!("unexpected error: {}", err),
            };

            http_server.expect(
                httptest::Expectation::matching(httptest::all_of![
                    httptest::matchers::request::method("PUT"),
                    httptest::matchers::request::path(format!("/{}/write", id)),
                    httptest::matchers::request::body(std::str::from_utf8(&BIG_DATA).unwrap()),
                ])
                .respond_with(httptest::responders::status_code(201)),
            );

            http_server.expect(
                httptest::Expectation::matching(httptest::all_of![
                    httptest::matchers::request::method("GET"),
                    httptest::matchers::request::path(format!("/{}/read", id)),
                ])
                .respond_with(
                    httptest::responders::status_code(200)
                        .body(std::str::from_utf8(&BIG_DATA).unwrap()),
                ),
            );

            let new_id = ContentWriterExt::write_content(&content_provider, &BIG_DATA)
                .await
                .unwrap();

            assert_eq!(new_id, id);

            let (data, origin) = content_provider
                .read_content_with_origin(&id)
                .await
                .unwrap();
            assert_eq!(&data, &BIG_DATA);
            assert_eq!(
                origin,
                Origin::Local {
                    path: "fake".into()
                }
            );

            // Make sure the next write yields `Error::AlreadyExists`.
            address_provider.set_already_exists(true).await;

            // Another write should be useless.
            match content_provider.get_content_writer(&id).await {
                Ok(_) => panic!("expected HashRefAlreadyExists error"),
                Err(Error::HashRefAlreadyExists(err_id)) => assert_eq!(err_id, id),
                Err(err) => panic!("unexpected error: {}", err),
            };
        }

        loop {
            tokio::select! {
                res = async {
                    server.serve_with_incoming(incoming).await
                } => panic!("server is no longer bound: {}", res.unwrap_err()),
                _ = f(&addr, &http_server, address_provider, data_space) => break
            };
        }
    }
}
