//! This file is heavily inspired by:
//! <https://github.com/hyperium/tonic/blob/master/tonic/src/transport/server/mod.rs>
//!
//! Tonic provides a way to multiplex several services behind the same transport/HTTP2 endpoint but
//! uses complex macro-definitions to do so. As a result, it is impossible to compose a multiplexed
//! `gRPC` server in a dynamic way that would allow several components to each register their own
//! service in their own time.
//!
//! Hence this module.

use std::{
    net::SocketAddr,
    task::{Context, Poll},
    time::Duration,
};

use bytes::Bytes;
use futures_util::future;
use http::{Request, Response};
use hyper::{server::conn::AddrStream, Body};
use log::debug;
use tower::Service;

/// An HTTP2 server transport for `gRPC` services.
///
/// Can be used to multiplex several `gRPC` services behind the same endpoint.
///
/// Routing can be done through the service name (compatible with Tonic's default behavior) or
/// explicitely.
#[derive(Default, Clone)]
pub struct Server {
    concurrency_limit: Option<usize>,
    timeout: Option<Duration>,
    init_stream_window_size: Option<u32>,
    init_connection_window_size: Option<u32>,
    max_concurrent_streams: Option<u32>,
    tcp_keepalive: Option<Duration>,
    tcp_nodelay: bool,
    http2_keepalive_interval: Option<Duration>,
    http2_keepalive_timeout: Option<Duration>,
    max_frame_size: Option<u32>,
    accept_http1: bool,

    router: Router,
}

const DEFAULT_HTTP2_KEEPALIVE_TIMEOUT_SECS: u64 = 20;

impl Server {
    /// Consume this [`Server`] creating a future that will execute the server on tokio's default
    /// executor.
    ///
    /// # Errors
    ///
    /// If the specified listening address cannot be bound-to, an error is returned.
    pub async fn serve(self, addr: &SocketAddr) -> anyhow::Result<()> {
        let concurrency_limit = self.concurrency_limit;
        let init_connection_window_size = self.init_connection_window_size;
        let init_stream_window_size = self.init_stream_window_size;
        let max_concurrent_streams = self.max_concurrent_streams;
        let timeout = self.timeout;
        let max_frame_size = self.max_frame_size;
        let http2_only = !self.accept_http1;
        let http2_keepalive_interval = self.http2_keepalive_interval;
        let http2_keepalive_timeout = self
            .http2_keepalive_timeout
            .unwrap_or_else(|| Duration::new(DEFAULT_HTTP2_KEEPALIVE_TIMEOUT_SECS, 0));

        let mut incoming = hyper::server::conn::AddrIncoming::bind(addr)?;

        incoming
            .set_keepalive(self.tcp_keepalive)
            .set_nodelay(self.tcp_nodelay);

        let server = hyper::Server::builder(incoming)
            .http2_only(http2_only)
            .http2_initial_connection_window_size(init_connection_window_size)
            .http2_initial_stream_window_size(init_stream_window_size)
            .http2_max_concurrent_streams(max_concurrent_streams)
            .http2_keep_alive_interval(http2_keepalive_interval)
            .http2_keep_alive_timeout(http2_keepalive_timeout)
            .http2_max_frame_size(max_frame_size);

        // Hyper HTTP2 server takes a "service" which receives a connection and returns a HTTP
        // service to serve requests on that connection.
        let service_maker = ServiceMaker {
            inner: self.router,
            concurrency_limit,
            timeout,
        };

        server.serve(service_maker).await?;

        Ok(())
    }
}

type BoxHttpBody = http_body::combinators::BoxBody<Bytes, anyhow::Error>;
type BoxService = tower::util::BoxService<Request<Body>, Response<BoxHttpBody>, tower::BoxError>;

struct ServiceMaker {
    concurrency_limit: Option<usize>,
    timeout: Option<Duration>,
    inner: Router,
}

impl Service<&AddrStream> for ServiceMaker {
    type Response = BoxService;
    type Error = anyhow::Error;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, _conn: &AddrStream) -> Self::Future {
        let concurrency_limit = self.concurrency_limit;
        let timeout = self.timeout;

        let svc = tower::ServiceBuilder::new()
            .layer(BoxService::layer())
            .option_layer(concurrency_limit.map(tower::limit::ConcurrencyLimitLayer::new))
            .option_layer(timeout.map(tower::timeout::TimeoutLayer::new))
            .service(self.inner.clone());

        future::ready(Ok(svc))
    }
}

////#[pin_project]
//struct SvcFuture<F> {
//    //#[pin]
//    inner: F,
//}
//
//impl<F, E, ResBody> Future for SvcFuture<F>
//where
//    F: Future<Output = Result<Response<ResBody>, E>>,
//    E: Into<anyhow::Error>,
//    ResBody: http_body::Body<Data = Bytes> + Send + Sync + 'static,
//    ResBody::Error: Into<anyhow::Error>,
//{
//    type Output = Result<Response<BoxHttpBody>, anyhow::Error>;
//
//    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//        let this = self.project();
//
//        let response: Response<ResBody> = ready!(this.inner.poll(cx)).map_err(Into::into)?;
//        let response = response.map(|body| body.map_err(Into::into).boxed());
//        Poll::Ready(Ok(response))
//    }
//}

#[derive(Default, Clone)]
pub struct Router {}

impl Service<Request<Body>> for Router {
    type Response = Response<BoxHttpBody>;
    type Error = anyhow::Error;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let path = req.uri().path();

        debug!("received service request for: {}", path);

        todo!()
    }
}
