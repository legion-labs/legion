use std::net::SocketAddr;

use api_codegen::cars::{
    client,
    models::{self, CarColor},
    requests::{
        CreateCarRequest, DeleteCarRequest, GetCarRequest, GetCarsRequest, TestBinaryRequest,
        TestOneOfRequest,
    },
    responses::{
        CreateCarResponse, DeleteCarResponse, GetCarResponse, GetCars200Response, GetCarsResponse,
        TestBinary200Response, TestBinaryResponse, TestOneOf200Response, TestOneOfResponse,
    },
    server, Api,
};
use axum::Router;
use lgn_online::codegen::Context;
use tokio::task::JoinHandle;

#[tokio::test]
async fn test_crud() -> anyhow::Result<()> {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let client = client::Client::new(hyper::Client::new(), format!("http://{}", addr));
    let handle = start_server(addr).await;

    let space_id = "ABCDEF".to_string();
    let span_id = "123456".to_string();
    let car = models::Car {
        id: 1,
        color: CarColor::Red,
        name: "car1".to_string(),
        is_new: true,
        extra: None,
    };

    let mut ctx = Context::default();
    let req = CreateCarRequest {
        space_id: space_id.clone(),
        span_id: Some(span_id),
        body: car.clone(),
    };
    let resp = client.create_car(&mut ctx, req).await?;
    assert_eq!(resp, CreateCarResponse::Status201);

    let mut ctx = Context::default();
    let req = GetCarsRequest {
        space_id: space_id.clone(),
        names: Some(vec!["car1".to_string()]),
        q: None,
    };
    let resp = client.get_cars(&mut ctx, req).await?;
    assert_eq!(
        resp,
        GetCarsResponse::Status200(GetCars200Response {
            body: vec![car.clone()]
        })
    );

    let req = GetCarRequest {
        space_id: space_id.clone(),
        car_id: 2,
    };
    let resp = client.get_car(&mut ctx, req).await?;
    assert_eq!(resp, GetCarResponse::Status404);

    let req = DeleteCarRequest {
        space_id: space_id.clone(),
        car_id: 1,
    };
    let resp = client.delete_car(&mut ctx, req).await.unwrap();
    assert_eq!(resp, DeleteCarResponse::Status200);

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn test_binary() -> anyhow::Result<()> {
    let addr = "127.0.0.1:3001".parse().unwrap();
    let client = client::Client::new(hyper::Client::new(), format!("http://{}", addr));
    let handle = start_server(addr).await;

    let mut ctx = Context::default();
    let req = TestBinaryRequest {
        space_id: "ABCDEF".to_string(),
        body: b"123456".to_vec().into(),
    };
    let resp = client.test_binary(&mut ctx, req).await?;
    assert_eq!(
        resp,
        TestBinaryResponse::Status200(TestBinary200Response {
            body: b"123456".to_vec().into()
        })
    );

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn test_one_of() -> anyhow::Result<()> {
    let addr = "127.0.0.1:3002".parse().unwrap();
    let client = client::Client::new(hyper::Client::new(), format!("http://{}", addr));
    let handle = start_server(addr).await;

    let mut ctx = Context::default();
    let req = TestOneOfRequest {};
    let resp = client.test_one_of(&mut ctx, req).await?;
    assert_eq!(
        resp,
        TestOneOfResponse::Status200(TestOneOf200Response {
            body: models::TestOneOfResponse::Option1(models::Pet {
                name: Some("Cat".to_string()),
            }),
        })
    );

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn test_context() -> anyhow::Result<()> {
    let addr = "127.0.0.1:3003".parse().unwrap();
    let client = client::Client::new(hyper::Client::new(), format!("http://{}", addr));
    let handle = start_server(addr).await;

    let mut extensions = http::Extensions::new();
    extensions.insert(5i32);

    let mut headers = http::HeaderMap::new();
    headers.insert("X-Test-Header", http::HeaderValue::from_static("test"));

    let mut ctx = Context::default();
    ctx.set_request_extensions(extensions);
    ctx.set_request_headers(headers);
    let req = TestOneOfRequest {};
    let resp = client.test_one_of(&mut ctx, req).await?;

    assert_eq!(
        resp,
        TestOneOfResponse::Status200(TestOneOf200Response {
            body: models::TestOneOfResponse::Option1(models::Pet {
                name: Some("Cat".to_string()),
            }),
        })
    );

    assert_eq!(
        ctx.response().unwrap().headers.get("X-Test-Header"),
        Some(&http::HeaderValue::from_static("test"))
    );

    handle.abort();
    Ok(())
}

async fn start_server(addr: SocketAddr) -> JoinHandle<Result<(), hyper::Error>> {
    let router = Router::new();
    let api = api_codegen::api_impl::ApiImpl::default();
    let router = server::register_routes(router, api);
    let server =
        axum::Server::bind(&addr).serve(router.into_make_service_with_connect_info::<SocketAddr>());

    tokio::spawn(async move { server.await })
}
