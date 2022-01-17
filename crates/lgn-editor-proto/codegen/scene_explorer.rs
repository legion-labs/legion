#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetEntityHierarchyRequest {
    #[prost(string, tag = "1")]
    pub filter: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub top_resource_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EntityInfo {
    #[prost(string, tag = "1")]
    pub path: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub entity_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub r#type: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub resource_id: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "5")]
    pub children: ::prost::alloc::vec::Vec<EntityInfo>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetEntityHierarchyResponse {
    #[prost(message, optional, tag = "1")]
    pub entity_info: ::core::option::Option<EntityInfo>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateEntityRequest {
    #[prost(string, optional, tag = "1")]
    pub entity_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub template_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub scene_resource_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub parent_resource_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "5")]
    pub init_values: ::prost::alloc::vec::Vec<InitPropertyValue>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InitPropertyValue {
    #[prost(string, tag = "1")]
    pub property_path: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub json_value: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateEntityResponse {
    #[prost(string, tag = "1")]
    pub new_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteEntitiesRequest {
    #[prost(string, repeated, tag = "1")]
    pub resource_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteEntitiesResponse {}
#[doc = r" Generated client implementations."]
pub mod scene_explorer_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct SceneExplorerClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl SceneExplorerClient<tonic::transport::Channel> {
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
    impl<T> SceneExplorerClient<T>
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
        ) -> SceneExplorerClient<InterceptedService<T, F>>
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
            SceneExplorerClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn create_entity(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateEntityRequest>,
        ) -> Result<tonic::Response<super::CreateEntityResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/scene_explorer.SceneExplorer/CreateEntity");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_entities(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteEntitiesRequest>,
        ) -> Result<tonic::Response<super::DeleteEntitiesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/scene_explorer.SceneExplorer/DeleteEntities",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_entity_hierarchy(
            &mut self,
            request: impl tonic::IntoRequest<super::GetEntityHierarchyRequest>,
        ) -> Result<tonic::Response<super::GetEntityHierarchyResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/scene_explorer.SceneExplorer/GetEntityHierarchy",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod scene_explorer_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with SceneExplorerServer."]
    #[async_trait]
    pub trait SceneExplorer: Send + Sync + 'static {
        async fn create_entity(
            &self,
            request: tonic::Request<super::CreateEntityRequest>,
        ) -> Result<tonic::Response<super::CreateEntityResponse>, tonic::Status>;
        async fn delete_entities(
            &self,
            request: tonic::Request<super::DeleteEntitiesRequest>,
        ) -> Result<tonic::Response<super::DeleteEntitiesResponse>, tonic::Status>;
        async fn get_entity_hierarchy(
            &self,
            request: tonic::Request<super::GetEntityHierarchyRequest>,
        ) -> Result<tonic::Response<super::GetEntityHierarchyResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct SceneExplorerServer<T: SceneExplorer> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: SceneExplorer> SceneExplorerServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for SceneExplorerServer<T>
    where
        T: SceneExplorer,
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
                "/scene_explorer.SceneExplorer/CreateEntity" => {
                    #[allow(non_camel_case_types)]
                    struct CreateEntitySvc<T: SceneExplorer>(pub Arc<T>);
                    impl<T: SceneExplorer> tonic::server::UnaryService<super::CreateEntityRequest>
                        for CreateEntitySvc<T>
                    {
                        type Response = super::CreateEntityResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateEntityRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_entity(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateEntitySvc(inner);
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
                "/scene_explorer.SceneExplorer/DeleteEntities" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteEntitiesSvc<T: SceneExplorer>(pub Arc<T>);
                    impl<T: SceneExplorer> tonic::server::UnaryService<super::DeleteEntitiesRequest>
                        for DeleteEntitiesSvc<T>
                    {
                        type Response = super::DeleteEntitiesResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteEntitiesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).delete_entities(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteEntitiesSvc(inner);
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
                "/scene_explorer.SceneExplorer/GetEntityHierarchy" => {
                    #[allow(non_camel_case_types)]
                    struct GetEntityHierarchySvc<T: SceneExplorer>(pub Arc<T>);
                    impl<T: SceneExplorer>
                        tonic::server::UnaryService<super::GetEntityHierarchyRequest>
                        for GetEntityHierarchySvc<T>
                    {
                        type Response = super::GetEntityHierarchyResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetEntityHierarchyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_entity_hierarchy(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetEntityHierarchySvc(inner);
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
    impl<T: SceneExplorer> Clone for SceneExplorerServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: SceneExplorer> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: SceneExplorer> tonic::transport::NamedService for SceneExplorerServer<T> {
        const NAME: &'static str = "scene_explorer.SceneExplorer";
    }
}
