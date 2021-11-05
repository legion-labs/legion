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
use log::{info, LevelFilter};
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
        Ok(Response::new(EchoResponse {
            msg: request.into_inner().msg,
        }))
    }
}

#[tonic::async_trait]
impl Summer for Service {
    async fn sum(&self, request: Request<SumRequest>) -> Result<Response<SumResponse>, Status> {
        let request = request.into_inner();

        Ok(Response::new(SumResponse {
            result: request.a + request.b,
        }))
    }
}

#[tokio::test]
async fn test_service_multiplexer() -> anyhow::Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();

    //let server = legion_grpc::server::transport::http2::Server::default();
    let echo_service = EchoerServer::new(Service {});
    let sum_service = SummerServer::new(Service {});
    let service = legion_grpc::service::multiplexer::MultiplexerService::builder()
        .add_service(echo_service)
        .add_service(sum_service)
        .build();

    assert!(service.is_some());

    let server = tonic::transport::Server::builder().add_optional_service(service);

    let addr = "127.0.0.1:50051".parse()?;

    async fn f() -> anyhow::Result<()> {
        let client = legion_grpc::client::Client::new_http(hyper::Uri::from_static(
            "http://127.0.0.1:50051",
        ));

        {
            let msg: String = "hello".into();

            let resp = backoff::future::retry(ExponentialBackoff::default(), || async {
                let mut echo_client = EchoerClient::new(client.clone());

                Ok(echo_client
                    .echo(Request::new(EchoRequest { msg: msg.clone() }))
                    .await?)
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

                Ok(sum_client.sum(Request::new(SumRequest { a, b })).await?)
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
