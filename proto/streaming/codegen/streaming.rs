#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InitializeStreamRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub rtc_session_description: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InitializeStreamResponse {
    #[prost(oneof = "initialize_stream_response::Response", tags = "3, 4")]
    pub response: ::core::option::Option<initialize_stream_response::Response>,
}
/// Nested message and enum types in `InitializeStreamResponse`.
pub mod initialize_stream_response {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Ok {
        #[prost(bytes = "vec", tag = "1")]
        pub rtc_session_description: ::prost::alloc::vec::Vec<u8>,
        #[prost(string, tag = "2")]
        pub stream_id: ::prost::alloc::string::String,
    }
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Response {
        #[prost(message, tag = "3")]
        Ok(Ok),
        #[prost(string, tag = "4")]
        Error(::prost::alloc::string::String),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddIceCandidatesRequest {
    #[prost(string, tag = "1")]
    pub stream_id: ::prost::alloc::string::String,
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub ice_candidates: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddIceCandidatesResponse {
    #[prost(bool, tag = "1")]
    pub ok: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IceCandidateRequest {
    #[prost(string, tag = "1")]
    pub stream_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IceCandidateResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub ice_candidate: ::prost::alloc::vec::Vec<u8>,
}
#[doc = r" Generated client implementations."]
pub mod streamer_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct StreamerClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl StreamerClient<tonic::transport::Channel> {
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
    impl<T> StreamerClient<T>
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
        ) -> StreamerClient<InterceptedService<T, F>>
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
            StreamerClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn initialize_stream(
            &mut self,
            request: impl tonic::IntoRequest<super::InitializeStreamRequest>,
        ) -> Result<tonic::Response<super::InitializeStreamResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/streaming.Streamer/InitializeStream");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn add_ice_candidates(
            &mut self,
            request: impl tonic::IntoRequest<super::AddIceCandidatesRequest>,
        ) -> Result<tonic::Response<super::AddIceCandidatesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/streaming.Streamer/AddIceCandidates");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn ice_candidates(
            &mut self,
            request: impl tonic::IntoRequest<super::IceCandidateRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::IceCandidateResponse>>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/streaming.Streamer/IceCandidates");
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod streamer_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with StreamerServer."]
    #[async_trait]
    pub trait Streamer: Send + Sync + 'static {
        async fn initialize_stream(
            &self,
            request: tonic::Request<super::InitializeStreamRequest>,
        ) -> Result<tonic::Response<super::InitializeStreamResponse>, tonic::Status>;
        async fn add_ice_candidates(
            &self,
            request: tonic::Request<super::AddIceCandidatesRequest>,
        ) -> Result<tonic::Response<super::AddIceCandidatesResponse>, tonic::Status>;
        #[doc = "Server streaming response type for the IceCandidates method."]
        type IceCandidatesStream: futures_core::Stream<Item = Result<super::IceCandidateResponse, tonic::Status>>
            + Send
            + 'static;
        async fn ice_candidates(
            &self,
            request: tonic::Request<super::IceCandidateRequest>,
        ) -> Result<tonic::Response<Self::IceCandidatesStream>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct StreamerServer<T: Streamer> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Streamer> StreamerServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for StreamerServer<T>
    where
        T: Streamer,
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
                "/streaming.Streamer/InitializeStream" => {
                    #[allow(non_camel_case_types)]
                    struct InitializeStreamSvc<T: Streamer>(pub Arc<T>);
                    impl<T: Streamer> tonic::server::UnaryService<super::InitializeStreamRequest>
                        for InitializeStreamSvc<T>
                    {
                        type Response = super::InitializeStreamResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::InitializeStreamRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).initialize_stream(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = InitializeStreamSvc(inner);
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
                "/streaming.Streamer/AddIceCandidates" => {
                    #[allow(non_camel_case_types)]
                    struct AddIceCandidatesSvc<T: Streamer>(pub Arc<T>);
                    impl<T: Streamer> tonic::server::UnaryService<super::AddIceCandidatesRequest>
                        for AddIceCandidatesSvc<T>
                    {
                        type Response = super::AddIceCandidatesResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AddIceCandidatesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).add_ice_candidates(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AddIceCandidatesSvc(inner);
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
                "/streaming.Streamer/IceCandidates" => {
                    #[allow(non_camel_case_types)]
                    struct IceCandidatesSvc<T: Streamer>(pub Arc<T>);
                    impl<T: Streamer>
                        tonic::server::ServerStreamingService<super::IceCandidateRequest>
                        for IceCandidatesSvc<T>
                    {
                        type Response = super::IceCandidateResponse;
                        type ResponseStream = T::IceCandidatesStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::IceCandidateRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).ice_candidates(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = IceCandidatesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.server_streaming(method, req).await;
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
    impl<T: Streamer> Clone for StreamerServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Streamer> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Streamer> tonic::transport::NamedService for StreamerServer<T> {
        const NAME: &'static str = "streaming.Streamer";
    }
}
