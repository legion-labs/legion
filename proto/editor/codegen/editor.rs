#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UndoTransactionRequest {
    #[prost(int32, tag = "1")]
    pub id: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UndoTransactionResponse {
    #[prost(int32, tag = "1")]
    pub id: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RedoTransactionRequest {
    #[prost(int32, tag = "1")]
    pub id: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RedoTransactionResponse {
    #[prost(int32, tag = "1")]
    pub id: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchResourcesRequest {
    #[prost(string, tag = "1")]
    pub search_token: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchResourcesResponse {
    #[prost(string, tag = "1")]
    pub next_search_token: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub total: u64,
    #[prost(message, repeated, tag = "3")]
    pub resource_descriptions: ::prost::alloc::vec::Vec<ResourceDescription>,
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
pub struct GetResourcePropertiesRequest {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
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
    #[prost(bytes = "vec", tag = "3")]
    pub default_value: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "5")]
    pub group: ::prost::alloc::string::String,
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
pub struct UpdateResourcePropertiesResponse {
    #[prost(uint32, tag = "1")]
    pub version: u32,
    #[prost(message, repeated, tag = "2")]
    pub updated_properties: ::prost::alloc::vec::Vec<ResourcePropertyUpdate>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourcePropertyUpdate {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
#[doc = r" Generated client implementations."]
pub mod editor_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct EditorClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl EditorClient<tonic::transport::Channel> {
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
    impl<T> EditorClient<T>
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
        ) -> EditorClient<InterceptedService<T, F>>
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
            EditorClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn search_resources(
            &mut self,
            request: impl tonic::IntoRequest<super::SearchResourcesRequest>,
        ) -> Result<tonic::Response<super::SearchResourcesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/editor.Editor/SearchResources");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn undo_transaction(
            &mut self,
            request: impl tonic::IntoRequest<super::UndoTransactionRequest>,
        ) -> Result<tonic::Response<super::UndoTransactionResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/editor.Editor/UndoTransaction");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn redo_transaction(
            &mut self,
            request: impl tonic::IntoRequest<super::RedoTransactionRequest>,
        ) -> Result<tonic::Response<super::RedoTransactionResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/editor.Editor/RedoTransaction");
            self.inner.unary(request.into_request(), path, codec).await
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
            let path = http::uri::PathAndQuery::from_static("/editor.Editor/GetResourceProperties");
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
            let path =
                http::uri::PathAndQuery::from_static("/editor.Editor/UpdateResourceProperties");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod editor_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with EditorServer."]
    #[async_trait]
    pub trait Editor: Send + Sync + 'static {
        async fn search_resources(
            &self,
            request: tonic::Request<super::SearchResourcesRequest>,
        ) -> Result<tonic::Response<super::SearchResourcesResponse>, tonic::Status>;
        async fn undo_transaction(
            &self,
            request: tonic::Request<super::UndoTransactionRequest>,
        ) -> Result<tonic::Response<super::UndoTransactionResponse>, tonic::Status>;
        async fn redo_transaction(
            &self,
            request: tonic::Request<super::RedoTransactionRequest>,
        ) -> Result<tonic::Response<super::RedoTransactionResponse>, tonic::Status>;
        async fn get_resource_properties(
            &self,
            request: tonic::Request<super::GetResourcePropertiesRequest>,
        ) -> Result<tonic::Response<super::GetResourcePropertiesResponse>, tonic::Status>;
        async fn update_resource_properties(
            &self,
            request: tonic::Request<super::UpdateResourcePropertiesRequest>,
        ) -> Result<tonic::Response<super::UpdateResourcePropertiesResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct EditorServer<T: Editor> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Editor> EditorServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for EditorServer<T>
    where
        T: Editor,
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
                "/editor.Editor/SearchResources" => {
                    #[allow(non_camel_case_types)]
                    struct SearchResourcesSvc<T: Editor>(pub Arc<T>);
                    impl<T: Editor> tonic::server::UnaryService<super::SearchResourcesRequest>
                        for SearchResourcesSvc<T>
                    {
                        type Response = super::SearchResourcesResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SearchResourcesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).search_resources(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SearchResourcesSvc(inner);
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
                "/editor.Editor/UndoTransaction" => {
                    #[allow(non_camel_case_types)]
                    struct UndoTransactionSvc<T: Editor>(pub Arc<T>);
                    impl<T: Editor> tonic::server::UnaryService<super::UndoTransactionRequest>
                        for UndoTransactionSvc<T>
                    {
                        type Response = super::UndoTransactionResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UndoTransactionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).undo_transaction(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UndoTransactionSvc(inner);
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
                "/editor.Editor/RedoTransaction" => {
                    #[allow(non_camel_case_types)]
                    struct RedoTransactionSvc<T: Editor>(pub Arc<T>);
                    impl<T: Editor> tonic::server::UnaryService<super::RedoTransactionRequest>
                        for RedoTransactionSvc<T>
                    {
                        type Response = super::RedoTransactionResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RedoTransactionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).redo_transaction(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RedoTransactionSvc(inner);
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
                "/editor.Editor/GetResourceProperties" => {
                    #[allow(non_camel_case_types)]
                    struct GetResourcePropertiesSvc<T: Editor>(pub Arc<T>);
                    impl<T: Editor> tonic::server::UnaryService<super::GetResourcePropertiesRequest>
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
                "/editor.Editor/UpdateResourceProperties" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateResourcePropertiesSvc<T: Editor>(pub Arc<T>);
                    impl<T: Editor>
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
    impl<T: Editor> Clone for EditorServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Editor> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Editor> tonic::transport::NamedService for EditorServer<T> {
        const NAME: &'static str = "editor.Editor";
    }
}
