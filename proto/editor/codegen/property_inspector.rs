#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetResourcePropertiesRequest {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourceDescription {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub path: ::prost::alloc::string::String,
    #[prost(uint32, tag = "3")]
    pub version: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetResourcePropertiesResponse {
    #[prost(message, optional, tag = "1")]
    pub description: ::core::option::Option<ResourceDescription>,
    #[prost(message, repeated, tag = "2")]
    pub properties: ::prost::alloc::vec::Vec<ResourceProperty>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourceProperty {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub ptype: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "4")]
    pub json_value: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "5")]
    pub sub_properties: ::prost::alloc::vec::Vec<ResourceProperty>,
    #[prost(map = "string, string", tag = "6")]
    pub attributes:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateResourcePropertiesRequest {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(uint32, tag = "2")]
    pub version: u32,
    #[prost(message, repeated, tag = "3")]
    pub property_updates: ::prost::alloc::vec::Vec<ResourcePropertyUpdate>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateResourcePropertiesResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourcePropertyUpdate {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub json_value: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteArrayElementRequest {
    #[prost(string, tag = "1")]
    pub resource_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub array_path: ::prost::alloc::string::String,
    #[prost(uint64, repeated, tag = "3")]
    pub indices: ::prost::alloc::vec::Vec<u64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteArrayElementResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InsertNewArrayElementRequest {
    #[prost(string, tag = "1")]
    pub resource_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub array_path: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub index: u64,
    #[prost(string, tag = "4")]
    pub json_value: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InsertNewArrayElementResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReorderArrayElementRequest {
    #[prost(string, tag = "1")]
    pub resource_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub array_path: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub old_index: u64,
    #[prost(uint64, tag = "4")]
    pub new_index: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReorderArrayElementResponse {}
#[doc = r" Generated client implementations."]
pub mod property_inspector_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct PropertyInspectorClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl PropertyInspectorClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> PropertyInspectorClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> PropertyInspectorClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            PropertyInspectorClient::new(InterceptedService::new(inner, interceptor))
        }
        #[doc = r" Compress requests with `gzip`."]
        #[doc = r""]
        #[doc = r" This requires the server to support it otherwise it might respond with an"]
        #[doc = r" error."]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        #[doc = r" Enable decompressing responses with `gzip`."]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        pub async fn get_resource_properties(
            &mut self,
            request: impl tonic::IntoRequest<super::GetResourcePropertiesRequest>,
        ) -> Result<tonic::Response<super::GetResourcePropertiesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/property_inspector.PropertyInspector/GetResourceProperties",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_resource_properties(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateResourcePropertiesRequest>,
        ) -> Result<tonic::Response<super::UpdateResourcePropertiesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/property_inspector.PropertyInspector/UpdateResourceProperties",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_array_element(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteArrayElementRequest>,
        ) -> Result<tonic::Response<super::DeleteArrayElementResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/property_inspector.PropertyInspector/DeleteArrayElement",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn insert_new_array_element(
            &mut self,
            request: impl tonic::IntoRequest<super::InsertNewArrayElementRequest>,
        ) -> Result<tonic::Response<super::InsertNewArrayElementResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/property_inspector.PropertyInspector/InsertNewArrayElement",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn reorder_array_element(
            &mut self,
            request: impl tonic::IntoRequest<super::ReorderArrayElementRequest>,
        ) -> Result<tonic::Response<super::ReorderArrayElementResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/property_inspector.PropertyInspector/ReorderArrayElement",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod property_inspector_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with PropertyInspectorServer."]
    #[async_trait]
    pub trait PropertyInspector: Send + Sync + 'static {
        async fn get_resource_properties(
            &self,
            request: tonic::Request<super::GetResourcePropertiesRequest>,
        ) -> Result<tonic::Response<super::GetResourcePropertiesResponse>, tonic::Status>;
        async fn update_resource_properties(
            &self,
            request: tonic::Request<super::UpdateResourcePropertiesRequest>,
        ) -> Result<tonic::Response<super::UpdateResourcePropertiesResponse>, tonic::Status>;
        async fn delete_array_element(
            &self,
            request: tonic::Request<super::DeleteArrayElementRequest>,
        ) -> Result<tonic::Response<super::DeleteArrayElementResponse>, tonic::Status>;
        async fn insert_new_array_element(
            &self,
            request: tonic::Request<super::InsertNewArrayElementRequest>,
        ) -> Result<tonic::Response<super::InsertNewArrayElementResponse>, tonic::Status>;
        async fn reorder_array_element(
            &self,
            request: tonic::Request<super::ReorderArrayElementRequest>,
        ) -> Result<tonic::Response<super::ReorderArrayElementResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct PropertyInspectorServer<T: PropertyInspector> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: PropertyInspector> PropertyInspectorServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for PropertyInspectorServer<T>
    where
        T: PropertyInspector,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/property_inspector.PropertyInspector/GetResourceProperties" => {
                    #[allow(non_camel_case_types)]
                    struct GetResourcePropertiesSvc<T: PropertyInspector>(pub Arc<T>);
                    impl<T: PropertyInspector>
                        tonic::server::UnaryService<super::GetResourcePropertiesRequest>
                        for GetResourcePropertiesSvc<T>
                    {
                        type Response = super::GetResourcePropertiesResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetResourcePropertiesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).get_resource_properties(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetResourcePropertiesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/property_inspector.PropertyInspector/UpdateResourceProperties" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateResourcePropertiesSvc<T: PropertyInspector>(pub Arc<T>);
                    impl<T: PropertyInspector>
                        tonic::server::UnaryService<super::UpdateResourcePropertiesRequest>
                        for UpdateResourcePropertiesSvc<T>
                    {
                        type Response = super::UpdateResourcePropertiesResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateResourcePropertiesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).update_resource_properties(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateResourcePropertiesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/property_inspector.PropertyInspector/DeleteArrayElement" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteArrayElementSvc<T: PropertyInspector>(pub Arc<T>);
                    impl<T: PropertyInspector>
                        tonic::server::UnaryService<super::DeleteArrayElementRequest>
                        for DeleteArrayElementSvc<T>
                    {
                        type Response = super::DeleteArrayElementResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteArrayElementRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).delete_array_element(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteArrayElementSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/property_inspector.PropertyInspector/InsertNewArrayElement" => {
                    #[allow(non_camel_case_types)]
                    struct InsertNewArrayElementSvc<T: PropertyInspector>(pub Arc<T>);
                    impl<T: PropertyInspector>
                        tonic::server::UnaryService<super::InsertNewArrayElementRequest>
                        for InsertNewArrayElementSvc<T>
                    {
                        type Response = super::InsertNewArrayElementResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::InsertNewArrayElementRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).insert_new_array_element(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = InsertNewArrayElementSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/property_inspector.PropertyInspector/ReorderArrayElement" => {
                    #[allow(non_camel_case_types)]
                    struct ReorderArrayElementSvc<T: PropertyInspector>(pub Arc<T>);
                    impl<T: PropertyInspector>
                        tonic::server::UnaryService<super::ReorderArrayElementRequest>
                        for ReorderArrayElementSvc<T>
                    {
                        type Response = super::ReorderArrayElementResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ReorderArrayElementRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).reorder_array_element(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ReorderArrayElementSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: PropertyInspector> Clone for PropertyInspectorServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: PropertyInspector> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: PropertyInspector> tonic::transport::NamedService for PropertyInspectorServer<T> {
        const NAME: &'static str = "property_inspector.PropertyInspector";
    }
}
