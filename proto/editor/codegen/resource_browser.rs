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
pub struct GetResourceTypeNamesRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetResourceTypeNamesResponse {
    #[prost(string, repeated, tag = "1")]
    pub resource_types: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateResourceRequest {
    #[prost(string, tag = "1")]
    pub resource_type: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub resource_path: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateResourceResponse {
    #[prost(string, tag = "1")]
    pub new_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImportResourceRequest {
    #[prost(string, tag = "1")]
    pub resource_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub shared_file_path: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImportResourceResponse {
    #[prost(string, tag = "1")]
    pub new_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteResourceRequest {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteResourceResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RenameResourceRequest {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub new_path: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RenameResourceResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CloneResourceRequest {
    #[prost(string, tag = "1")]
    pub source_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub clone_path: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CloneResourceResponse {
    #[prost(string, tag = "1")]
    pub new_id: ::prost::alloc::string::String,
}
#[doc = r" Generated client implementations."]
pub mod resource_browser_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct ResourceBrowserClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ResourceBrowserClient<tonic::transport::Channel> {
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
    impl<T> ResourceBrowserClient<T>
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
        ) -> ResourceBrowserClient<InterceptedService<T, F>>
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
            ResourceBrowserClient::new(InterceptedService::new(inner, interceptor))
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
            let path = http::uri::PathAndQuery::from_static(
                "/resource_browser.ResourceBrowser/SearchResources",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_resource(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateResourceRequest>,
        ) -> Result<tonic::Response<super::CreateResourceResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/resource_browser.ResourceBrowser/CreateResource",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_resource_type_names(
            &mut self,
            request: impl tonic::IntoRequest<super::GetResourceTypeNamesRequest>,
        ) -> Result<tonic::Response<super::GetResourceTypeNamesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/resource_browser.ResourceBrowser/GetResourceTypeNames",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn import_resource(
            &mut self,
            request: impl tonic::IntoRequest<super::ImportResourceRequest>,
        ) -> Result<tonic::Response<super::ImportResourceResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/resource_browser.ResourceBrowser/ImportResource",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_resource(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteResourceRequest>,
        ) -> Result<tonic::Response<super::DeleteResourceResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/resource_browser.ResourceBrowser/DeleteResource",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn rename_resource(
            &mut self,
            request: impl tonic::IntoRequest<super::RenameResourceRequest>,
        ) -> Result<tonic::Response<super::RenameResourceResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/resource_browser.ResourceBrowser/RenameResource",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn clone_resource(
            &mut self,
            request: impl tonic::IntoRequest<super::CloneResourceRequest>,
        ) -> Result<tonic::Response<super::CloneResourceResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/resource_browser.ResourceBrowser/CloneResource",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod resource_browser_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with ResourceBrowserServer."]
    #[async_trait]
    pub trait ResourceBrowser: Send + Sync + 'static {
        async fn search_resources(
            &self,
            request: tonic::Request<super::SearchResourcesRequest>,
        ) -> Result<tonic::Response<super::SearchResourcesResponse>, tonic::Status>;
        async fn create_resource(
            &self,
            request: tonic::Request<super::CreateResourceRequest>,
        ) -> Result<tonic::Response<super::CreateResourceResponse>, tonic::Status>;
        async fn get_resource_type_names(
            &self,
            request: tonic::Request<super::GetResourceTypeNamesRequest>,
        ) -> Result<tonic::Response<super::GetResourceTypeNamesResponse>, tonic::Status>;
        async fn import_resource(
            &self,
            request: tonic::Request<super::ImportResourceRequest>,
        ) -> Result<tonic::Response<super::ImportResourceResponse>, tonic::Status>;
        async fn delete_resource(
            &self,
            request: tonic::Request<super::DeleteResourceRequest>,
        ) -> Result<tonic::Response<super::DeleteResourceResponse>, tonic::Status>;
        async fn rename_resource(
            &self,
            request: tonic::Request<super::RenameResourceRequest>,
        ) -> Result<tonic::Response<super::RenameResourceResponse>, tonic::Status>;
        async fn clone_resource(
            &self,
            request: tonic::Request<super::CloneResourceRequest>,
        ) -> Result<tonic::Response<super::CloneResourceResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ResourceBrowserServer<T: ResourceBrowser> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: ResourceBrowser> ResourceBrowserServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ResourceBrowserServer<T>
    where
        T: ResourceBrowser,
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
                "/resource_browser.ResourceBrowser/SearchResources" => {
                    #[allow(non_camel_case_types)]
                    struct SearchResourcesSvc<T: ResourceBrowser>(pub Arc<T>);
                    impl<T: ResourceBrowser>
                        tonic::server::UnaryService<super::SearchResourcesRequest>
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
                "/resource_browser.ResourceBrowser/CreateResource" => {
                    #[allow(non_camel_case_types)]
                    struct CreateResourceSvc<T: ResourceBrowser>(pub Arc<T>);
                    impl<T: ResourceBrowser>
                        tonic::server::UnaryService<super::CreateResourceRequest>
                        for CreateResourceSvc<T>
                    {
                        type Response = super::CreateResourceResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateResourceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_resource(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateResourceSvc(inner);
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
                "/resource_browser.ResourceBrowser/GetResourceTypeNames" => {
                    #[allow(non_camel_case_types)]
                    struct GetResourceTypeNamesSvc<T: ResourceBrowser>(pub Arc<T>);
                    impl<T: ResourceBrowser>
                        tonic::server::UnaryService<super::GetResourceTypeNamesRequest>
                        for GetResourceTypeNamesSvc<T>
                    {
                        type Response = super::GetResourceTypeNamesResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetResourceTypeNamesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).get_resource_type_names(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetResourceTypeNamesSvc(inner);
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
                "/resource_browser.ResourceBrowser/ImportResource" => {
                    #[allow(non_camel_case_types)]
                    struct ImportResourceSvc<T: ResourceBrowser>(pub Arc<T>);
                    impl<T: ResourceBrowser>
                        tonic::server::UnaryService<super::ImportResourceRequest>
                        for ImportResourceSvc<T>
                    {
                        type Response = super::ImportResourceResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ImportResourceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).import_resource(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ImportResourceSvc(inner);
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
                "/resource_browser.ResourceBrowser/DeleteResource" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteResourceSvc<T: ResourceBrowser>(pub Arc<T>);
                    impl<T: ResourceBrowser>
                        tonic::server::UnaryService<super::DeleteResourceRequest>
                        for DeleteResourceSvc<T>
                    {
                        type Response = super::DeleteResourceResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteResourceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).delete_resource(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteResourceSvc(inner);
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
                "/resource_browser.ResourceBrowser/RenameResource" => {
                    #[allow(non_camel_case_types)]
                    struct RenameResourceSvc<T: ResourceBrowser>(pub Arc<T>);
                    impl<T: ResourceBrowser>
                        tonic::server::UnaryService<super::RenameResourceRequest>
                        for RenameResourceSvc<T>
                    {
                        type Response = super::RenameResourceResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RenameResourceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).rename_resource(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RenameResourceSvc(inner);
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
                "/resource_browser.ResourceBrowser/CloneResource" => {
                    #[allow(non_camel_case_types)]
                    struct CloneResourceSvc<T: ResourceBrowser>(pub Arc<T>);
                    impl<T: ResourceBrowser>
                        tonic::server::UnaryService<super::CloneResourceRequest>
                        for CloneResourceSvc<T>
                    {
                        type Response = super::CloneResourceResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CloneResourceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).clone_resource(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CloneResourceSvc(inner);
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
    impl<T: ResourceBrowser> Clone for ResourceBrowserServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: ResourceBrowser> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ResourceBrowser> tonic::transport::NamedService for ResourceBrowserServer<T> {
        const NAME: &'static str = "resource_browser.ResourceBrowser";
    }
}
