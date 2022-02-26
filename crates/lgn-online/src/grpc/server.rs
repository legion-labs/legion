use std::{net::SocketAddr, sync::Arc};

use http::{Request, Response};
use lgn_tracing::info;
use tokio::sync::Mutex;
use tonic::{body::BoxBody, transport::NamedService};
use tower::Service;

use super::{Error, Result};
use crate::aws::lambda::is_running_as_lambda;

#[derive(Default)]
pub struct Server {
    listen_address: Option<SocketAddr>,
}

impl Server {
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
        S::Error: Into<lambda_runtime::Error> + Send,
    {
        let service = tonic_web::enable(service);

        match ExecutionEnvironment::guess() {
            ExecutionEnvironment::AWSLambda => {
                let service = &Arc::new(Mutex::new(service));

                let handler =
                    lambda_http::service_fn(move |event: lambda_http::Request| async move {
                        let request = event.map(|b| b.to_vec().into());

                        let response = service
                            .lock()
                            .await
                            .call(request)
                            .await
                            .map_err(Into::into)?;

                        let (parts, body) = response.into_parts();
                        let body = hyper::body::to_bytes(body).await?.to_vec();
                        Ok(lambda_http::Response::from_parts(parts, body))
                    });

                info!(
                    "AWS Lambda execution environment detected: starting gRPC-web server as lambda..."
                );

                lambda_http::run(handler)
                    .await
                    .map_err(Into::into)
                    .map_err(Error::Other)
            }
            ExecutionEnvironment::Local => {
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
    }
}

pub enum ExecutionEnvironment {
    AWSLambda,
    Local,
}

impl ExecutionEnvironment {
    pub fn guess() -> Self {
        if is_running_as_lambda() {
            Self::AWSLambda
        } else {
            Self::Local
        }
    }
}
