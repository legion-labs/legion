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
use tonic::body::BoxBody;
use tower::Service;

/// An HTTP2 server transport for `gRPC` services.
///
/// Can be used to multiplex several `gRPC` services behind the same endpoint.
///
/// Routing can be done through the service name (compatible with Tonic's default behavior) or
/// explicitely.
#[derive(Default)]
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

    services: Services,
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
        let root_service = MakeSvc {
            inner: self.services,
            concurrency_limit,
            timeout,
        };

        server.serve(root_service).await?;

        Ok(())
    }
}

type BoxHttpBody = http_body::combinators::BoxBody<Bytes, anyhow::Error>;
type BoxService = tower::util::BoxService<Request<Body>, Response<BoxHttpBody>, anyhow::Error>;

struct MakeSvc<S> {
    concurrency_limit: Option<usize>,
    timeout: Option<Duration>,
    inner: S,
}

impl<S, ResBody> Service<&AddrStream> for MakeSvc<S>
where
    S: Service<Request<Body>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<anyhow::Error> + Send,
    ResBody: http_body::Body<Data = Bytes> + Send + Sync + 'static,
    ResBody::Error: Into<anyhow::Error>,
{
    type Response = BoxService;
    type Error = anyhow::Error;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, io: &AddrStream) -> Self::Future {
        //let conn_info = io.connect_info();

        //let svc = self.inner.clone();
        //let concurrency_limit = self.concurrency_limit;
        //let timeout = self.timeout;

        //let svc = ServiceBuilder::new()
        //    .option_layer(concurrency_limit.map(ConcurrencyLimitLayer::new))
        //    .layer_fn(|s| GrpcTimeout::new(s, timeout))
        //    .service(svc);

        //let svc = ServiceBuilder::new()
        //    .layer(BoxService::layer())
        //    .map_request(move |mut request: Request<Body>| {
        //        match &conn_info {
        //            tower::util::Either::A(inner) => {
        //                request.extensions_mut().insert(inner.clone());
        //            }
        //            tower::util::Either::B(inner) => {
        //                #[cfg(feature = "tls")]
        //                {
        //                    request.extensions_mut().insert(inner.clone());
        //                    request.extensions_mut().insert(inner.get_ref().clone());
        //                }

        //                #[cfg(not(feature = "tls"))]
        //                {
        //                    // just a type check to make sure we didn't forget to
        //                    // insert this into the extensions
        //                    let _: &() = inner;
        //                }
        //            }
        //        }

        //        request
        //    })
        //    .service(Svc { inner: svc });

        //future::ready(Ok(svc))
        future::ready(Err(anyhow::format_err!("foo")))
    }
}

#[derive(Default, Clone)]
pub struct Services {}

impl Service<Request<Body>> for Services {
    type Response = Response<BoxBody>;

    type Error = anyhow::Error;

    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        todo!()
    }
}
