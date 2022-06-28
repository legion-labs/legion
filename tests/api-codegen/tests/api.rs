use std::net::SocketAddr;

use api_codegen::api::cars::{
    client::{
        Client, CreateCarRequest, CreateCarResponse, DeleteCarRequest, DeleteCarResponse,
        GetCarRequest, GetCarResponse, GetCarsRequest, GetCarsResponse, TestBinaryRequest,
        TestBinaryResponse, TestHeadersRequest, TestHeadersResponse, TestOneOfResponse,
    },
    server, TestOneOf200Response,
};
use api_codegen::api::components::{Alpha, Beta, Car, CarColor, Cars, Gamma, Pet};
use axum::Router;
use http::{header::HeaderName, HeaderValue};
use lgn_online::client::HyperClient;
use tokio::task::JoinHandle;

#[tokio::test]
async fn test_crud() {
    let addr = "127.0.0.1:3100".parse().unwrap();
    let handle = start_server(addr).await;
    let client = new_client(addr);

    let space_id = "ABCDEF".to_string();
    let span_id = "123456".to_string();
    let car = Car {
        id: 1,
        code: 65,
        color: CarColor::Red,
        name: "car1".to_string(),
        is_new: true,
        extra: None,
    };

    let req = CreateCarRequest {
        space_id: space_id.clone(),
        span_id: Some(span_id),
        body: car.clone(),
    };
    let resp = client.create_car(req).await.unwrap();
    assert!(matches!(resp, CreateCarResponse::Status201 { .. }));

    let req = GetCarsRequest {
        space_id: space_id.clone(),
        names: Some(vec!["car1".to_string(), "car2".to_string()]),
        q: None,
        other_query: "other_query".to_string(),
    };
    let resp = client.get_cars(req).await.unwrap();

    match resp {
        GetCarsResponse::Status200 { body, .. } => {
            assert_eq!(body, Cars(vec![car.clone()]));
        }
    };

    let req = GetCarRequest {
        space_id: space_id.clone(),
        car_id: 2,
    };
    let resp = client.get_car(req).await.unwrap();
    assert!(matches!(resp, GetCarResponse::Status404 { .. }));

    let req = DeleteCarRequest {
        space_id: space_id.clone(),
        car_id: 1,
    };
    let resp = client.delete_car(req).await.unwrap();
    assert!(matches!(resp, DeleteCarResponse::Status200 { .. }));

    handle.abort();
}

#[tokio::test]
async fn test_binary() {
    let addr = "127.0.0.1:3002".parse().unwrap();
    let handle = start_server(addr).await;
    let client = new_client(addr);

    let req = TestBinaryRequest {
        space_id: "ABCDEF".to_string(),
        body: b"123456".to_vec().into(),
    };
    let resp = client.test_binary(req).await.unwrap();

    match resp {
        TestBinaryResponse::Status200 { body, .. } => assert_eq!(body, b"123456".to_vec().into()),
    };

    handle.abort();
}

#[tokio::test]
async fn test_one_of() {
    let addr = "127.0.0.1:3003".parse().unwrap();
    let handle = start_server(addr).await;
    let client = new_client(addr);

    let resp = client.test_one_of().await.unwrap();
    match resp {
        TestOneOfResponse::Status200 { body, .. } => assert_eq!(
            body,
            TestOneOf200Response::Option3(Alpha {
                beta: Some(Beta(vec![Gamma::Option1(Box::new(Alpha { beta: None }),)])),
            }),
        ),
    };

    handle.abort();
}

#[tokio::test]
async fn test_headers() {
    let addr = "127.0.0.1:3004".parse().unwrap();
    let handle = start_server(addr).await;
    let client = HyperClient::default();
    let client = tower_http::add_extension::AddExtension::new(client, 5i32);
    let client = tower_http::set_header::SetRequestHeader::if_not_present(
        client,
        HeaderName::from_static("x-dyn-header"),
        HeaderValue::from_str("dyn").unwrap(),
    );
    let client = Client::new(client, format!("http://{}", addr).parse().unwrap());

    let req = TestHeadersRequest {
        x_string_header: Some("string".to_string()),
        x_int_header: Some(5),
        x_bytes_header: Some(b"bytes".to_vec().into()),
    };
    let resp = client.test_headers(req).await.unwrap();

    match resp {
        TestHeadersResponse::Status200 {
            x_string_header,
            x_int_header,
            x_bytes_header,
            body,
            mut extra_headers,
            ..
        } => {
            assert_eq!(x_string_header, "string".to_string());
            assert_eq!(x_int_header, 5);
            assert_eq!(x_bytes_header, b"bytes".to_vec().into());
            assert_eq!(
                body,
                Pet {
                    name: Some("Cat".to_string()),
                }
            );
            assert_eq!(
                extra_headers.remove("X-Dyn-Header"),
                Some(http::HeaderValue::from_static("dyn"))
            );
        }
    };

    handle.abort();
}

async fn start_server(addr: SocketAddr) -> JoinHandle<Result<(), hyper::Error>> {
    let api = api_codegen::api_impl::ApiImpl::default();
    let router = server::register_routes(Router::new(), api);
    let server =
        axum::Server::bind(&addr).serve(router.into_make_service_with_connect_info::<SocketAddr>());

    tokio::spawn(async move { server.await })
}

fn new_client(addr: SocketAddr) -> Client<HyperClient> {
    Client::new(
        HyperClient::default(),
        format!("http://{}", addr).parse().unwrap(),
    )
}
