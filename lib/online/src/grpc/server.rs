use std::net::SocketAddr;

use http::{Request, Response};
use lambda_http::handler;
use lgn_telemetry::info;
use tonic::{body::BoxBody, transport::NamedService};
use tower::Service;

use super::{aws_lambda_handler::AwsLambdaHandler, Error, Result};
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
                let handler = handler(AwsLambdaHandler::new(service));

                info!(
                    "AWS Lambda execution environment detected: starting gRPC-web server as lambda..."
                );

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
