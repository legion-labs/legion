use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use crate::api::cars::{
    self,
    requests::{
        CreateCarRequest, DeleteCarRequest, GetCarRequest, GetCarsRequest, TestBinaryRequest,
        TestHeadersRequest,
    },
    responses::{
        CreateCarResponse, DeleteCarResponse, GetCarResponse, GetCarsResponse, TestBinaryResponse,
        TestHeadersResponse, TestOneOfResponse,
    },
    Api,
};
use crate::api::components;
use axum::extract::ConnectInfo;
use lgn_online::server::Result;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Default)]
pub struct ApiImpl {
    cars: Arc<RwLock<HashMap<i64, components::Car>>>,
}

#[async_trait::async_trait]
impl Api for ApiImpl {
    async fn get_cars(
        &self,
        parts: http::request::Parts,
        _request: GetCarsRequest,
    ) -> Result<GetCarsResponse> {
        let connect_info = parts.extensions.get::<ConnectInfo<SocketAddr>>().unwrap();
        println!("Request addr: {}", connect_info.0);

        Ok(GetCarsResponse::Status200(components::Cars(
            self.cars.read().await.values().cloned().collect(),
        )))
    }

    async fn get_car(
        &self,
        _parts: http::request::Parts,
        request: GetCarRequest,
    ) -> Result<GetCarResponse> {
        let car = self.cars.read().await.get(&request.car_id).cloned();
        match car {
            Some(car) => Ok(GetCarResponse::Status200(car)),
            None => Ok(GetCarResponse::Status404),
        }
    }

    async fn create_car(
        &self,
        _parts: http::request::Parts,
        request: CreateCarRequest,
    ) -> Result<CreateCarResponse> {
        self.cars
            .write()
            .await
            .insert(request.body.id, request.body.clone());
        Ok(CreateCarResponse::Status201)
    }

    async fn delete_car(
        &self,
        _parts: http::request::Parts,
        request: DeleteCarRequest,
    ) -> Result<DeleteCarResponse> {
        self.cars.write().await.remove(&request.car_id);
        Ok(DeleteCarResponse::Status200)
    }

    async fn test_binary(
        &self,
        _parts: http::request::Parts,
        request: TestBinaryRequest,
    ) -> Result<TestBinaryResponse> {
        Ok(TestBinaryResponse::Status200(request.body))
    }

    async fn test_one_of(&self, _parts: http::request::Parts) -> Result<TestOneOfResponse> {
        Ok(TestOneOfResponse::Status200(
            cars::TestOneOf200Response::Option1(components::Pet {
                name: Some("Cat".to_string()),
            }),
        ))
    }

    async fn test_headers(
        &self,
        mut parts: http::request::Parts,
        request: TestHeadersRequest,
    ) -> Result<TestHeadersResponse> {
        let resp = TestHeadersResponse::Status200 {
            x_string_header: request.x_string_header.unwrap(),
            x_bytes_header: request.x_bytes_header.unwrap(),
            x_int_header: request.x_int_header.unwrap(),
            body: components::Pet {
                name: Some("Cat".to_string()),
            },
        };

        Ok(if let Some(value) = parts.headers.remove("x-dyn-header") {
            resp.with_header("x-dyn-header", value)
        } else {
            resp
        })
    }
}
