pub mod echo {
    tonic::include_proto!("echo");
}

pub mod sum {
    tonic::include_proto!("sum");
}

use echo::{
    echoer_client::EchoerClient,
    echoer_server::{Echoer, EchoerServer},
    EchoRequest, EchoResponse,
};

use sum::{
    summer_client::SummerClient,
    summer_server::{Summer, SummerServer},
    SumRequest, SumResponse,
};

use log::LevelFilter;
use simple_logger::SimpleLogger;
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
async fn test_http2_server() -> anyhow::Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();

    //let server = legion_grpc::server::transport::http2::Server::default();
    let echo_service = EchoerServer::new(Service {});
    let sum_service = SummerServer::new(Service {});
    let server = tonic::transport::Server::builder()
        .add_service(echo_service)
        .add_service(sum_service);

    let addr = "[::]:50051".parse()?;

    async fn f() -> anyhow::Result<()> {
        let client = hyper::Client::builder().http2_only(true).build_http();
        let uri = hyper::Uri::from_static("http://[::1]:50051");

        let add_origin = tower::service_fn(|mut req: hyper::Request<tonic::body::BoxBody>| {
            let uri = hyper::Uri::builder()
                .scheme(uri.scheme().unwrap().clone())
                .authority(uri.authority().unwrap().clone())
                .path_and_query(req.uri().path_and_query().unwrap().clone())
                .build()
                .unwrap();

            *req.uri_mut() = uri;

            client.request(req)
        });

        {
            let mut echo_client = EchoerClient::new(add_origin);

            let msg: String = "hello".into();
            let resp = echo_client
                .echo(Request::new(EchoRequest { msg: msg.clone() }))
                .await?;

            assert_eq!(resp.into_inner().msg, msg);
        }

        {
            let mut sum_client = SummerClient::new(add_origin);

            let a = 1;
            let b = 2;
            let result = 3;
            let resp = sum_client.sum(Request::new(SumRequest { a, b })).await?;

            assert_eq!(resp.into_inner().result, result);
        }

        Ok(())
    }

    loop {
        tokio::select! {
            res = server.serve(addr) => panic!("server is no longer bound: {}", res.unwrap_err()),
            res = f() => match res {
                Ok(_) => break,
                Err(err) => panic!("client execution failed: {}", err),
            },
        };
    }

    Ok(())
}
