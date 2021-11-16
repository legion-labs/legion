pub mod echo {
    tonic::include_proto!("echo");
}

pub mod sum {
    tonic::include_proto!("sum");
}

use backoff::ExponentialBackoff;
use echo::{
    echoer_client::EchoerClient,
    echoer_server::{Echoer, EchoerServer},
    EchoRequest, EchoResponse,
};
use legion_online::grpc::{GrpcClient, GrpcWebClient};
use log::{error, info, LevelFilter};
use simple_logger::SimpleLogger;
use sum::{
    summer_client::SummerClient,
    summer_server::{Summer, SummerServer},
    SumRequest, SumResponse,
};
use tonic::{Request, Response, Status};

struct Service {}

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

#[cfg(test)]
static INIT: std::sync::Once = std::sync::Once::new();

#[cfg(test)]
fn setup_test_logger() {
    INIT.call_once(|| {
        SimpleLogger::new()
            .with_level(LevelFilter::Debug)
            .init()
            .unwrap();
    });
}

#[tokio::test]
#[serial_test::serial]
async fn test_service_multiplexer() -> anyhow::Result<()> {
    setup_test_logger();

    //let server = legion_grpc::server::transport::http2::Server::default();
    let echo_service = EchoerServer::new(Service {});
    let sum_service = SummerServer::new(Service {});
    let service = legion_online::grpc::MultiplexerService::builder()
        .add_service(echo_service)
        .add_service(sum_service)
        .build();

    assert!(service.is_some());

    let server = tonic::transport::Server::builder().add_optional_service(service);

    let addr = "127.0.0.1:50051".parse()?;

    async fn f() -> anyhow::Result<()> {
        let client = GrpcClient::new("http://127.0.0.1:50051".parse()?);

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
            res = f() => match res {
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
    setup_test_logger();

    let echo_service = EchoerServer::new(Service {});

    let server = tonic::transport::Server::builder().add_service(echo_service);

    let addr = "127.0.0.1:50051".parse()?;

    async fn f() -> anyhow::Result<()> {
        let client = GrpcClient::new("http://127.0.0.1:50051".parse()?);

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
            res = f() => match res {
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
    setup_test_logger();

    let echo_service = EchoerServer::new(Service {});

    let server = tonic::transport::Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(echo_service));

    let addr = "127.0.0.1:50051".parse()?;

    async fn f() -> anyhow::Result<()> {
        let client = GrpcWebClient::new("http://127.0.0.1:50051".parse()?);

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
            res = f() => match res {
                Ok(_) => break,
                Err(err) => panic!("client execution failed: {}", err),
            },
        };
    }

    Ok(())
}
