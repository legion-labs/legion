pub mod echo {
    tonic::include_proto!("echo");
}

use echo::{
    echoer_client::EchoerClient,
    echoer_server::{Echoer, EchoerServer},
    EchoRequest, EchoResponse,
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

#[tokio::test]
async fn test_http2_server() -> anyhow::Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();

    let server = legion_grpc::server::transport::http2::Server::default();
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

        let mut echo_client = EchoerClient::new(add_origin);

        let msg: String = "hello".into();
        let resp = echo_client
            .echo(Request::new(EchoRequest { msg: msg.clone() }))
            .await?;

        assert_eq!(resp.into_inner().msg, msg);

        Ok(())
    }

    loop {
        tokio::select! {
            res = server.serve(&addr) => panic!("server is no longer bound: {}", res.unwrap_err()),
            res = f() => match res {
                Ok(_) => break,
                Err(err) => panic!("client execution failed: {}", err),
            },
        };
    }

    Ok(())
}
