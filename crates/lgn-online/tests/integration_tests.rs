#[allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::similar_names,
    clippy::use_self
)]
pub mod echo {
    tonic::include_proto!("echo");
}

#[allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::similar_names,
    clippy::use_self
)]
pub mod sum {
    tonic::include_proto!("sum");
}

use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use backoff::ExponentialBackoff;
use echo::{
    echoer_client::EchoerClient,
    echoer_server::{Echoer, EchoerServer},
    EchoRequest, EchoResponse,
};
use lgn_online::{
    authentication::{self, Authenticator, ClientTokenSet},
    grpc::{AuthenticatedClient, GrpcClient, GrpcWebClient},
};
use lgn_tracing::{error, info};
use sum::{
    summer_client::SummerClient,
    summer_server::{Summer, SummerServer},
    SumRequest, SumResponse,
};
use tonic::{Request, Response, Status};

struct Service {}

fn get_random_localhost_addr() -> String {
    match std::net::TcpListener::bind("127.0.0.1:0") {
        Ok(stream) => format!("127.0.0.1:{}", stream.local_addr().unwrap().port()),
        Err(_) => "127.0.0.1:50051".to_string(),
    }
}

#[tonic::async_trait]
impl Echoer for Service {
    async fn echo(&self, request: Request<EchoRequest>) -> Result<Response<EchoResponse>, Status> {
        let request = request.into_inner();

        info!("Got a request: {:?}", request);

        Ok(Response::new(EchoResponse { msg: request.msg }))
    }
}

#[tonic::async_trait]
impl Summer for Service {
    async fn sum(&self, request: Request<SumRequest>) -> Result<Response<SumResponse>, Status> {
        let request = request.into_inner();

        info!("Got a request: {:?}", request);

        Ok(Response::new(SumResponse {
            result: request.a + request.b,
        }))
    }
}

#[derive(Default, Clone)]
struct MockAuthenticator {}

#[async_trait]
impl Authenticator for MockAuthenticator {
    async fn login(
        &self,
        _scopes: &[String],
        _extra_params: &Option<HashMap<String, String>>,
    ) -> authentication::Result<ClientTokenSet> {
        Ok(ClientTokenSet {
            access_token: "access_token".into(),
            refresh_token: None,
            id_token: Some("id_token".into()),
            token_type: "token_type".into(),
            expires_in: Some(Duration::new(123456789, 0)),
            scopes: None,
        })
    }

    async fn refresh_login(
        &self,
        _client_token_set: ClientTokenSet,
    ) -> authentication::Result<ClientTokenSet> {
        self.login(&[], &None).await
    }

    async fn logout(&self) -> authentication::Result<()> {
        Ok(())
    }
}

#[tokio::test]
#[serial_test::serial]
async fn test_service_multiplexer() -> anyhow::Result<()> {
    //let server = lgn_grpc::server::transport::http2::Server::default();
    let echo_service = EchoerServer::new(Service {});
    let sum_service = SummerServer::new(Service {});
    let service = lgn_online::grpc::MultiplexerService::builder()
        .add_service(echo_service)
        .add_service(sum_service)
        .build();

    assert!(service.is_some());

    let server = tonic::transport::Server::builder().add_optional_service(service);

    let addr_str = get_random_localhost_addr();
    let addr = addr_str.parse()?;

    async fn f(addr_str: &str) -> anyhow::Result<()> {
        let client = GrpcClient::new(format!("http://{}", addr_str).parse()?);

        let authenticator = MockAuthenticator::default();
        let client = AuthenticatedClient::new(client, authenticator, &[]);

        {
            let msg: String = "hello".into();

            let resp = backoff::future::retry(ExponentialBackoff::default(), || async {
                let mut echo_client = EchoerClient::new(client.clone());

                let resp = echo_client
                    .echo(Request::new(EchoRequest { msg: msg.clone() }))
                    .await;

                if let Err(e) = &resp {
                    error!("unexpected error: {}", e);
                }

                Ok(resp?)
            })
            .await?;

            assert_eq!(resp.into_inner().msg, msg);
        }

        {
            let a = 1;
            let b = 2;
            let result = 3;
            let resp = backoff::future::retry(ExponentialBackoff::default(), || async {
                let mut sum_client = SummerClient::new(client.clone());

                let resp = sum_client.sum(Request::new(SumRequest { a, b })).await;

                if let Err(e) = &resp {
                    error!("unexpected error: {}", e);
                }

                Ok(resp?)
            })
            .await?;

            assert_eq!(resp.into_inner().result, result);
        }

        Ok(())
    }

    loop {
        tokio::select! {
            res = async {
                info!("starting gRPC server...");

                server.serve(addr).await
            } => panic!("server is no longer bound: {}", res.unwrap_err()),
            res = f(&addr_str) => match res {
                Ok(_) => break,
                Err(err) => panic!("client execution failed: {}", err),
            },
        };
    }

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn test_http2_client_and_server() -> anyhow::Result<()> {
    let echo_service = EchoerServer::new(Service {});

    let server = tonic::transport::Server::builder().add_service(echo_service);

    let addr_str = get_random_localhost_addr();
    let addr = addr_str.parse()?;

    async fn f(addr_str: &str) -> anyhow::Result<()> {
        let client = GrpcClient::new(format!("http://{}", addr_str).parse()?);

        {
            let msg: String = "hello".into();

            let resp = backoff::future::retry(ExponentialBackoff::default(), || async {
                let mut echo_client = EchoerClient::new(client.clone());

                let resp = echo_client
                    .echo(Request::new(EchoRequest { msg: msg.clone() }))
                    .await;

                if let Err(e) = &resp {
                    error!("unexpected error: {}", e);
                }

                Ok(resp?)
            })
            .await?;

            assert_eq!(resp.into_inner().msg, msg);
        }

        Ok(())
    }

    loop {
        tokio::select! {
            res = async {
                info!("starting gRPC server...");

                server.serve(addr).await
            } => panic!("server is no longer bound: {}", res.unwrap_err()),
            res = f(&addr_str) => match res {
                Ok(_) => break,
                Err(err) => panic!("client execution failed: {}", err),
            },
        };
    }

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn test_http1_client_and_server() -> anyhow::Result<()> {
    let echo_service = EchoerServer::new(Service {});

    let server = tonic::transport::Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(echo_service));

    let addr_str = get_random_localhost_addr();
    let addr = addr_str.parse()?;

    async fn f(addr_str: &str) -> anyhow::Result<()> {
        let client = GrpcWebClient::new(format!("http://{}", addr_str).parse()?);

        {
            let msg: String = "hello".into();

            let resp = backoff::future::retry(ExponentialBackoff::default(), || async {
                let mut echo_client = EchoerClient::new(client.clone());

                let resp = echo_client
                    .echo(Request::new(EchoRequest { msg: msg.clone() }))
                    .await;

                if let Err(e) = &resp {
                    error!("unexpected error: {}", e);
                }

                Ok(resp?)
            })
            .await?;

            assert_eq!(resp.into_inner().msg, msg);
        }

        Ok(())
    }

    loop {
        tokio::select! {
            res = async {
                info!("starting gRPC server...");

                server.serve(addr).await
            } => panic!("server is no longer bound: {}", res.unwrap_err()),
            res = f(&addr_str) => match res {
                Ok(_) => break,
                Err(err) => panic!("client execution failed: {}", err),
            },
        };
    }

    Ok(())
}
