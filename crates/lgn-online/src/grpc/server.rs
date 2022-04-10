use std::{convert::Infallible, net::SocketAddr};

use http::{Request, Response};
use lgn_tracing::info;
use tonic::{body::BoxBody, transport::NamedService};
use tower::Service;

use super::{Error, Result};
#[cfg(feature = "aws")]
use crate::aws::lambda::{is_running_as_lambda, AwsLambdaHandler};

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
        S: Service<Request<hyper::Body>, Response = Response<BoxBody>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>> + Send,
    {
        let service = tonic_web::enable(service);

        match ExecutionEnvironment::guess() {
            #[cfg(feature = "aws")]
            ExecutionEnvironment::AWSLambda => {
                let handler = lambda_http::Adapter::from(AwsLambdaHandler::new(service));
                lambda_runtime::run(handler)
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
    #[cfg(feature = "aws")]
    AWSLambda,
    Local,
}

impl ExecutionEnvironment {
    pub fn guess() -> Self {
        #[cfg(feature = "aws")]
        if is_running_as_lambda() {
            Self::AWSLambda
        } else {
            Self::Local
        }

        #[cfg(not(feature = "aws"))]
        Self::Local
    }
}
