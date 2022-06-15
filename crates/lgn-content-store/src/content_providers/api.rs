use std::{
    fmt::{Debug, Display},
    pin::Pin,
    sync::Arc,
    task::Context,
    task::Poll,
};

use crate::api::content_store::client::{
    Client, GetContentWriterRequest, GetContentWriterResponse, ReadContentRequest,
    ReadContentResponse, WriteContentRequest, WriteContentResponse,
};
use async_trait::async_trait;
use futures::{stream::TryStreamExt, Future, FutureExt};
use http::{header, Uri};
use lgn_governance::types::SpaceId;
use lgn_tracing::{async_span_scope, debug, error, warn};
use pin_project::pin_project;
use tokio::{
    io::AsyncRead,
    io::{AsyncWrite, ReadBuf},
};
use tokio_util::{compat::FuturesAsyncReadCompatExt, io::ReaderStream};

use super::{
    ContentAsyncReadWithOriginAndSize, ContentAsyncWrite, ContentReader, ContentWriter, Error,
    HashRef, Result, Uploader, UploaderImpl, WithOriginAndSize,
};

use crate::DataSpace;

/// A `ApiContentProvider` is a provider that delegates to an `OpenApi` service.
#[derive(Debug, Clone)]
pub struct ApiContentProvider<C> {
    client: Arc<Client<C>>,
    space_id: SpaceId,
    data_space: DataSpace,
    buf_size: usize,
}

impl<C> ApiContentProvider<C> {
    pub async fn new(client: C, base_url: Uri, space_id: SpaceId, data_space: DataSpace) -> Self {
        let client = Arc::new(Client::new(client, base_url));
        // The buffer for HTTP uploaders is set to 2MB.
        let buf_size = 2 * 1024 * 1024;

        Self {
            client,
            space_id,
            data_space,
            buf_size,
        }
    }
}

impl<C> Display for ApiContentProvider<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "open api client (data space: {})", self.data_space)
    }
}

#[async_trait]
impl<C, ResBody> ContentReader for ApiContentProvider<C>
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
    async fn get_content_reader(&self, id: &HashRef) -> Result<ContentAsyncReadWithOriginAndSize> {
        async_span_scope!("ApiContentProvider::get_content_reader");

        debug!("ApiContentProvider::get_content_reader({})", id);

        let req = ReadContentRequest {
            space_id: self.space_id.clone().into(),
            data_space: self.data_space.clone().into(),
            content_id: id.into(),
        };

        let resp = self
            .client
            .read_content(req)
            .await
            .map_err(|err| anyhow::anyhow!("request failed: {}", err))?;

        match resp {
            ReadContentResponse::Status200 { body, x_origin, .. } => {
                debug!(
                    "ApiContentProvider::get_content_reader({}) -> content data is available",
                    id
                );

                let origin = rmp_serde::from_slice(&x_origin.0.to_vec())
                    .map_err(|err| anyhow::anyhow!("failed to parse origin: {}", err))?;

                Ok(std::io::Cursor::new(body.0).with_origin_and_size(origin, id.data_size()))
            }
            ReadContentResponse::Status204 {
                x_origin, x_url, ..
            } => {
                debug!(
                    "ApiContentProvider::get_content_reader({}) -> content URL is available",
                    id
                );

                let origin = rmp_serde::from_slice(&x_origin.0.to_vec())
                    .map_err(|err| anyhow::anyhow!("failed to parse origin: {}", err))?;

                Ok(HttpDownloader::new(x_url.0).with_origin_and_size(origin, id.data_size()))
            }
            ReadContentResponse::Status404 { .. } => {
                warn!(
                    "ApiContentProvider::get_content_reader({}) -> content does not exist",
                    id
                );

                Err(Error::HashRefNotFound(id.clone()))
            }
        }
    }
}

/// An `AsyncRead` instance that doesn't really connect to the specified URL until first polled.
#[pin_project]
struct HttpDownloader {
    #[pin]
    state: HttpDownloaderState,
}

enum HttpDownloaderState {
    Idle(String),
    Connecting(Pin<Box<dyn Future<Output = Result<reqwest::Response, reqwest::Error>> + Send>>),
    Reading(Pin<Box<dyn AsyncRead + Send>>),
}

impl HttpDownloader {
    fn new(url: String) -> Self {
        Self {
            state: HttpDownloaderState::Idle(url),
        }
    }
}

impl AsyncRead for HttpDownloader {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<tokio::io::Result<()>> {
        let this = self.project();
        let state = this.state.get_mut();

        loop {
            match state {
                HttpDownloaderState::Idle(url) => {
                    // The call hasn't started yet: we need to start it.
                    let call = Box::pin(reqwest::get(std::mem::take(url)));

                    // We need to poll it right away, as it's the only way to either
                    // move on to the next step directly, or to make sure the `cx`
                    // is notified when the call completes. To do so, we
                    // directly go to the next iteration after updating the
                    // state.

                    *state = HttpDownloaderState::Connecting(call);
                }
                HttpDownloaderState::Connecting(call) => {
                    match call.poll_unpin(cx) {
                        Poll::Ready(Ok(resp)) => {
                            match resp.error_for_status() {
                                Ok(resp) => {
                                    let reader = Box::pin(
                                        resp.bytes_stream()
                                            .map_err(|e| {
                                                futures::io::Error::new(
                                                    futures::io::ErrorKind::Other,
                                                    e,
                                                )
                                            })
                                            .into_async_read()
                                            .compat(),
                                    );

                                    *state = HttpDownloaderState::Reading(reader);

                                    // Okay we moved to the next step directly, so `cx`
                                    // won't be notified: let's run another iteration
                                    // straight away.
                                }
                                Err(err) => {
                                    error!(
                                            "HttpRequestAsyncRead::poll_read -> failed to read content from the specified URL: {}",
                                            err
                                        );

                                    return Poll::Ready(Err(tokio::io::Error::new(
                                        tokio::io::ErrorKind::Other,
                                        err,
                                    )));
                                }
                            }
                        }
                        Poll::Ready(Err(err)) => {
                            return Poll::Ready(Err(tokio::io::Error::new(
                                tokio::io::ErrorKind::Other,
                                err,
                            )));
                        }
                        Poll::Pending => {
                            return Poll::Pending;
                        }
                    }
                }
                HttpDownloaderState::Reading(reader) => {
                    tokio::pin!(reader);

                    return reader.poll_read(cx, buf);
                }
            }
        }
    }
}

#[async_trait]
impl<C, ResBody> ContentWriter for ApiContentProvider<C>
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
    async fn get_content_writer(&self, id: &HashRef) -> Result<ContentAsyncWrite> {
        async_span_scope!("ApiContentProvider::get_content_writer");

        let req = GetContentWriterRequest {
            space_id: self.space_id.clone().into(),
            data_space: self.data_space.clone().into(),
            content_id: id.into(),
        };

        let resp = self
            .client
            .get_content_writer(req)
            .await
            .map_err(|err| anyhow::anyhow!("request failed: {}", err))?;

        match resp {
            GetContentWriterResponse::Status200 { body, .. } => {
                if body.url.0.is_empty() {
                    Ok(Box::pin(OpenApiUploader::new(OpenApiUploaderImpl {
                        client: Arc::clone(&self.client),
                        space_id: self.space_id.clone(),
                        data_space: self.data_space.clone(),
                    })))
                } else {
                    let uploader = HttpUploader::new(id.clone(), body.url.0, self.buf_size);

                    Ok(Box::pin(uploader))
                }
            }
            GetContentWriterResponse::Status409 { .. } => {
                Err(Error::HashRefAlreadyExists(id.clone()))
            }
        }
    }
}

type OpenApiUploader<C> = Uploader<OpenApiUploaderImpl<C>>;

#[derive(Debug)]
struct OpenApiUploaderImpl<C> {
    client: Arc<Client<C>>,
    space_id: SpaceId,
    data_space: DataSpace,
}

#[async_trait]
impl<C, ResBody> UploaderImpl for OpenApiUploaderImpl<C>
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
    async fn upload(self, data: Vec<u8>) -> Result<()> {
        async_span_scope!("ApiContentProvider::upload");

        let id = HashRef::new_from_data(&data);
        let req = WriteContentRequest {
            space_id: self.space_id.into(),
            data_space: self.data_space.into(),
            body: data.into(),
        };

        let resp = self
            .client
            .write_content(req)
            .await
            .map_err(|err| anyhow::anyhow!("request failed: {}", err))?;

        match resp {
            WriteContentResponse::Status200 { body, .. } => {
                let res_id: HashRef = body.id.try_into()?;

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
    use axum::Router;
    use lgn_online::client::HyperClient;
    use lgn_online::server::RouterExt;
    use std::{collections::HashMap, net::SocketAddr};
    use tokio::sync::Mutex;

    use crate::{
        ApiProviderSet, ContentAddressReader, ContentAddressWriter, ContentReaderExt,
        ContentWriterExt, MemoryAliasProvider, MemoryContentProvider, Origin, Server,
    };

    use crate::api::content_store::server;

    use super::*;

    #[derive(Debug)]
    pub struct FakeContentAddressProvider {
        base_url: String,
        already_exists: Arc<Mutex<bool>>,
        origins: Arc<Mutex<HashMap<HashRef, Origin>>>,
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
                origins: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub async fn register_origin(&self, id: &HashRef, origin: Origin) {
            self.origins.lock().await.insert(id.clone(), origin);
        }

        pub fn get_address(&self, id: &HashRef, suffix: &str) -> String {
            format!("{}{}/{}", self.base_url, id, suffix)
        }

        pub async fn get_origin(&self, id: &HashRef) -> Result<Origin> {
            match self.origins.lock().await.get(id) {
                Some(origin) => Ok(origin.clone()),
                None => Err(Error::HashRefNotFound(id.clone())),
            }
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
            let address = self.get_address(id, "read");
            let origin = self.get_origin(id).await?;

            Ok((address, origin))
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

    #[tokio::test]
    async fn test_api_content_provider() {
        // To debug this test more easily, you may want to specify: RUST_LOG=httptest=debug
        let _ = pretty_env_logger::try_init();

        let _telemetry_guard = lgn_telemetry_sink::TelemetryGuardBuilder::default()
            .with_local_sink_max_level(lgn_tracing::LevelFilter::Debug)
            .build();

        let content_provider = MemoryContentProvider::new();
        let alias_provider = MemoryAliasProvider::new();

        let http_server = httptest::Server::run();

        const SMALL_DATA: [u8; 128] = [0x41; 128];
        const BIG_DATA: [u8; 512] = [0x41; 512];

        let address_provider = Arc::new(FakeContentAddressProvider::new(
            http_server.url("/").to_string(),
        ));
        let space_id = "space_id".parse().unwrap();
        let data_space = DataSpace::persistent();
        let providers = vec![(
            data_space.clone(),
            ApiProviderSet {
                content_provider: Box::new(content_provider),
                alias_provider: Box::new(alias_provider),
                content_address_provider: Box::new(Arc::clone(&address_provider)),
                size_threshold: SMALL_DATA.len(),
            },
        )]
        .into_iter()
        .collect();

        let service = Arc::new(Server::new(providers));
        let router = Router::new().apply_development_router_options();
        let router = server::register_routes(router, service);

        let addr = "127.0.0.1:0".parse().unwrap();
        let server = axum::Server::bind(&addr)
            .serve(router.into_make_service_with_connect_info::<SocketAddr>());
        let addr = server.local_addr();

        async fn f(
            socket_addr: &SocketAddr,
            http_server: &httptest::Server,
            address_provider: Arc<FakeContentAddressProvider>,
            space_id: SpaceId,
            data_space: DataSpace,
        ) {
            let client = HyperClient::default();
            let base_url: http::Uri = format!("http://{}", socket_addr).parse().unwrap();
            let content_provider =
                ApiContentProvider::new(client, base_url.clone(), space_id, data_space).await;

            let origin = Origin::Memory {};
            crate::content_providers::test_content_provider(&content_provider, &SMALL_DATA, origin)
                .await;

            // Now let's try again with a larger file to test the address lookup
            // & HTTP request mechanisms.

            let id = HashRef::new_from_data(&BIG_DATA);

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

            let expected_origin = Origin::Local {
                path: "fake".into(),
            };

            address_provider
                .register_origin(&id, expected_origin.clone())
                .await;

            let (data, origin) = content_provider
                .read_content_with_origin(&id)
                .await
                .unwrap();
            assert_eq!(&data, &BIG_DATA);
            assert_eq!(origin, expected_origin);

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
                    server
                    .with_graceful_shutdown(async move { lgn_cli_utils::wait_for_termination().await.unwrap() })
                    .await
                } => panic!("server is no longer bound: {}", res.unwrap_err()),
                _ = f(&addr, &http_server, address_provider, space_id, data_space) => break
            };
        }
    }
}
