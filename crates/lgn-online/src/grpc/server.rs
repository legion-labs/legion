use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use axum::Router;
use futures::future::try_join;
use http::{Request, Response};
use lgn_tracing::info;
use tonic::{body::BoxBody, transport::NamedService};
use tower::Service;

use crate::server::RouterExt;

use super::{Error, Result};

#[derive(Default)]
pub struct HybridServer {
    grpc_listen_address: Option<SocketAddr>,
    rest_listen_address: Option<SocketAddr>,
}

impl HybridServer {
    #[must_use]
    pub fn set_grpc_listen_address(mut self, listen_address: SocketAddr) -> Self {
        self.grpc_listen_address = Some(listen_address);

        self
    }

    #[must_use]
    pub fn set_rest_listen_address(mut self, listen_address: SocketAddr) -> Self {
        self.rest_listen_address = Some(listen_address);

        self
    }

    pub async fn run<S>(self, service: S, router: Arc<Mutex<Router>>) -> Result<()>
    where
        S: Service<Request<hyper::Body>, Response = Response<BoxBody>>
            + NamedService
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>> + Send,
    {
        let rest_listen_address = self.rest_listen_address.ok_or_else(|| {
            Error::RunServerFailure(
                "running as local server but no listen address was specified".to_string(),
            )
        })?;

        let rest_service = router
            .lock()
            .unwrap()
            .clone()
            .apply_development_router_options()
            .into_make_service_with_connect_info::<SocketAddr>();

        let rest_server = axum::Server::bind(&rest_listen_address).serve(rest_service);

        let grpc_listen_address = self.grpc_listen_address.ok_or_else(|| {
            Error::RunServerFailure(
                "running as local server but no listen address was specified".to_string(),
            )
        })?;

        let grpc_service = tonic_web::enable(service);

        let grpc_server = tonic::transport::Server::builder()
            .accept_http1(true)
            .add_service(grpc_service)
            .serve(grpc_listen_address);

        info!(
            "Starting rest web server at {} and gRpc web server at {}...",
            rest_listen_address, grpc_listen_address
        );

        try_join(
            async move { grpc_server.await.map_err(|err| Error::Other(err.into())) },
            async move { rest_server.await.map_err(|err| Error::Other(err.into())) },
        )
        .await
        .map(|_| ())
    }
}

#[derive(Default)]
pub struct Server {
    listen_address: Option<SocketAddr>,
}

impl Server {
    #[must_use]
    pub fn set_listen_address(mut self, listen_address: SocketAddr) -> Self {
        self.listen_address = Some(listen_address);

        self
    }

    pub async fn run<S>(self, service: S) -> Result<()>
    where
        S: Service<Request<hyper::Body>, Response = Response<BoxBody>>
            + NamedService
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>> + Send,
    {
        let service = tonic_web::enable(service);
        let listen_address = self.listen_address.ok_or_else(|| {
            Error::RunServerFailure(
                "running as local server but no listen address was specified".to_string(),
            )
        })?;

        info!("Starting local gRPC-web server at {}...", listen_address);

        tonic::transport::Server::builder()
            .accept_http1(true)
            .add_service(service)
            .serve(listen_address)
            .await
            .map_err(Into::into)
            .map_err(Error::Other)
    }
}
