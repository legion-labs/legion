use std::{
    collections::HashMap,
    task::{Context, Poll},
};

use dyn_clone::DynClone;
use log::{debug, info};
use tonic::transport::NamedService;

pub trait MultiplexableService: DynClone {
    fn call(
        &mut self,
        req: http::Request<hyper::Body>,
    ) -> BoxFuture<http::Response<tonic::body::BoxBody>, tonic::codegen::Never>;
}

dyn_clone::clone_trait_object!(MultiplexableService);

// Blanket implementation for all services: makes it possible to use all tonic-generated services
// in the `Multiplexer` service.
impl<S> MultiplexableService for S
where
    S: tower::Service<http::Request<hyper::Body>> + Clone,
    S::Future: Into<BoxFuture<http::Response<tonic::body::BoxBody>, tonic::codegen::Never>>,
{
    fn call(
        &mut self,
        req: http::Request<hyper::Body>,
    ) -> BoxFuture<http::Response<tonic::body::BoxBody>, tonic::codegen::Never> {
        S::call(self, req).into()
    }
}

type BoxMultiplexableService = Box<dyn MultiplexableService + Send>;

#[derive(Default)]
pub struct Multiplexer {
    services: HashMap<&'static str, BoxMultiplexableService>,
}

impl Multiplexer {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn add_service<S>(mut self, s: S) -> Self
    where
        S: MultiplexableService + NamedService + Send + 'static,
    {
        info!("registered service `{}`", S::NAME);

        self.services.insert(S::NAME, Box::new(s));

        self
    }
}

impl Clone for Multiplexer {
    fn clone(&self) -> Self {
        Self {
            services: self.services.clone(),
        }
    }
}

type BoxFuture<T, E> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send + 'static>>;

impl tower::Service<http::Request<hyper::Body>> for Multiplexer {
    type Response = http::Response<tonic::body::BoxBody>;

    type Error = tonic::codegen::Never;

    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<hyper::Body>) -> Self::Future {
        if let Some(svc_name) = req.uri().path().splitn(3, '/').nth(1) {
            match &mut self.services.get_mut(svc_name) {
                Some(svc) => {
                    debug!("dispatching call to service `{}`", svc_name);

                    svc.call(req)
                }
                None => {
                    debug!("dispatching call to service `{}`", svc_name);
                    Box::pin(futures_util::future::ok(
                        http::Response::builder()
                            .status(200)
                            .header("grpc-status", "12")
                            .header("content-type", "application/grpc")
                            .body(tonic::body::empty_body())
                            .unwrap(),
                    ))
                }
            }
        } else {
            debug!(
                "failed to dispatch call to service: the request path does not seem to contain a service name ({})",
                req.uri().path(),
            );

            Box::pin(futures_util::future::ok(
                http::Response::builder()
                    .status(200)
                    .header("grpc-status", "12")
                    .header("content-type", "application/grpc")
                    .body(tonic::body::empty_body())
                    .unwrap(),
            ))
        }
    }
}

impl NamedService for Multiplexer {
    const NAME: &'static str = "";
}
