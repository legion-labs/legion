/// find_process
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FindProcessRequest {
    #[prost(string, tag = "1")]
    pub process_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FindProcessReply {
    #[prost(message, optional, tag = "1")]
    pub process: ::core::option::Option<super::telemetry::Process>,
}
/// list_recent_processes
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RecentProcessesRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProcessInstance {
    #[prost(message, optional, tag = "1")]
    pub process_info: ::core::option::Option<super::telemetry::Process>,
    #[prost(uint32, tag = "2")]
    pub nb_cpu_blocks: u32,
    #[prost(uint32, tag = "3")]
    pub nb_log_blocks: u32,
    #[prost(uint32, tag = "4")]
    pub nb_metric_blocks: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProcessListReply {
    #[prost(message, repeated, tag = "1")]
    pub processes: ::prost::alloc::vec::Vec<ProcessInstance>,
}
/// list_process_streams
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListProcessStreamsRequest {
    #[prost(string, tag = "1")]
    pub process_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListStreamsReply {
    #[prost(message, repeated, tag = "1")]
    pub streams: ::prost::alloc::vec::Vec<super::telemetry::Stream>,
}
/// list_stream_blocks
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListStreamBlocksRequest {
    #[prost(string, tag = "1")]
    pub stream_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListStreamBlocksReply {
    #[prost(message, repeated, tag = "1")]
    pub blocks: ::prost::alloc::vec::Vec<super::telemetry::Block>,
}
/// block_spans
/// Span: represents a function call instance
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Span {
    #[prost(uint32, tag = "1")]
    pub scope_hash: u32,
    /// how many function calls are above this one in the thread
    #[prost(uint32, tag = "2")]
    pub depth: u32,
    #[prost(double, tag = "3")]
    pub begin_ms: f64,
    #[prost(double, tag = "4")]
    pub end_ms: f64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScopeDesc {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub filename: ::prost::alloc::string::String,
    #[prost(uint32, tag = "3")]
    pub line: u32,
    #[prost(uint32, tag = "4")]
    pub hash: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockSpansRequest {
    #[prost(message, optional, tag = "1")]
    pub process: ::core::option::Option<super::telemetry::Process>,
    #[prost(message, optional, tag = "2")]
    pub stream: ::core::option::Option<super::telemetry::Stream>,
    #[prost(string, tag = "3")]
    pub block_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockSpansReply {
    #[prost(message, repeated, tag = "1")]
    pub scopes: ::prost::alloc::vec::Vec<ScopeDesc>,
    #[prost(message, repeated, tag = "2")]
    pub spans: ::prost::alloc::vec::Vec<Span>,
    #[prost(string, tag = "3")]
    pub block_id: ::prost::alloc::string::String,
    #[prost(double, tag = "4")]
    pub begin_ms: f64,
    #[prost(double, tag = "5")]
    pub end_ms: f64,
    #[prost(uint32, tag = "6")]
    pub max_depth: u32,
}
/// process_cumulative_call_graph
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProcessCumulativeCallGraphRequest {
    #[prost(message, optional, tag = "1")]
    pub process: ::core::option::Option<super::telemetry::Process>,
    #[prost(double, tag = "2")]
    pub begin_ms: f64,
    #[prost(double, tag = "3")]
    pub end_ms: f64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeStats {
    #[prost(double, tag = "1")]
    pub sum: f64,
    #[prost(double, tag = "2")]
    pub min: f64,
    #[prost(double, tag = "3")]
    pub max: f64,
    #[prost(double, tag = "4")]
    pub avg: f64,
    #[prost(double, tag = "5")]
    pub median: f64,
    #[prost(uint64, tag = "6")]
    pub count: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CallGraphEdge {
    #[prost(uint32, tag = "1")]
    pub hash: u32,
    #[prost(double, tag = "2")]
    pub weight: f64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CumulativeCallGraphNode {
    #[prost(uint32, tag = "1")]
    pub hash: u32,
    #[prost(message, optional, tag = "2")]
    pub stats: ::core::option::Option<NodeStats>,
    #[prost(message, repeated, tag = "3")]
    pub callers: ::prost::alloc::vec::Vec<CallGraphEdge>,
    #[prost(message, repeated, tag = "4")]
    pub callees: ::prost::alloc::vec::Vec<CallGraphEdge>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CumulativeCallGraphReply {
    #[prost(message, repeated, tag = "1")]
    pub scopes: ::prost::alloc::vec::Vec<ScopeDesc>,
    #[prost(message, repeated, tag = "2")]
    pub nodes: ::prost::alloc::vec::Vec<CumulativeCallGraphNode>,
}
/// list_process_log_entries
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProcessLogRequest {
    #[prost(message, optional, tag = "1")]
    pub process: ::core::option::Option<super::telemetry::Process>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LogEntry {
    #[prost(double, tag = "1")]
    pub time_ms: f64,
    #[prost(string, tag = "2")]
    pub msg: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProcessLogReply {
    #[prost(message, repeated, tag = "1")]
    pub entries: ::prost::alloc::vec::Vec<LogEntry>,
}
/// list_process_children
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListProcessChildrenRequest {
    #[prost(string, tag = "1")]
    pub process_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProcessChildrenReply {
    #[prost(message, repeated, tag = "1")]
    pub processes: ::prost::alloc::vec::Vec<super::telemetry::Process>,
}
#[doc = r" Generated client implementations."]
pub mod performance_analytics_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct PerformanceAnalyticsClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl PerformanceAnalyticsClient<tonic::transport::Channel> {
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
    impl<T> PerformanceAnalyticsClient<T>
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
        ) -> PerformanceAnalyticsClient<InterceptedService<T, F>>
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
            PerformanceAnalyticsClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn block_spans(
            &mut self,
            request: impl tonic::IntoRequest<super::BlockSpansRequest>,
        ) -> Result<tonic::Response<super::BlockSpansReply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/analytics.PerformanceAnalytics/block_spans");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn process_cumulative_call_graph(
            &mut self,
            request: impl tonic::IntoRequest<super::ProcessCumulativeCallGraphRequest>,
        ) -> Result<tonic::Response<super::CumulativeCallGraphReply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/analytics.PerformanceAnalytics/process_cumulative_call_graph",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn find_process(
            &mut self,
            request: impl tonic::IntoRequest<super::FindProcessRequest>,
        ) -> Result<tonic::Response<super::FindProcessReply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/analytics.PerformanceAnalytics/find_process",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_process_children(
            &mut self,
            request: impl tonic::IntoRequest<super::ListProcessChildrenRequest>,
        ) -> Result<tonic::Response<super::ProcessChildrenReply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/analytics.PerformanceAnalytics/list_process_children",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_process_log_entries(
            &mut self,
            request: impl tonic::IntoRequest<super::ProcessLogRequest>,
        ) -> Result<tonic::Response<super::ProcessLogReply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/analytics.PerformanceAnalytics/list_process_log_entries",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_process_streams(
            &mut self,
            request: impl tonic::IntoRequest<super::ListProcessStreamsRequest>,
        ) -> Result<tonic::Response<super::ListStreamsReply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/analytics.PerformanceAnalytics/list_process_streams",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_recent_processes(
            &mut self,
            request: impl tonic::IntoRequest<super::RecentProcessesRequest>,
        ) -> Result<tonic::Response<super::ProcessListReply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/analytics.PerformanceAnalytics/list_recent_processes",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_stream_blocks(
            &mut self,
            request: impl tonic::IntoRequest<super::ListStreamBlocksRequest>,
        ) -> Result<tonic::Response<super::ListStreamBlocksReply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/analytics.PerformanceAnalytics/list_stream_blocks",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod performance_analytics_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with PerformanceAnalyticsServer."]
    #[async_trait]
    pub trait PerformanceAnalytics: Send + Sync + 'static {
        async fn block_spans(
            &self,
            request: tonic::Request<super::BlockSpansRequest>,
        ) -> Result<tonic::Response<super::BlockSpansReply>, tonic::Status>;
        async fn process_cumulative_call_graph(
            &self,
            request: tonic::Request<super::ProcessCumulativeCallGraphRequest>,
        ) -> Result<tonic::Response<super::CumulativeCallGraphReply>, tonic::Status>;
        async fn find_process(
            &self,
            request: tonic::Request<super::FindProcessRequest>,
        ) -> Result<tonic::Response<super::FindProcessReply>, tonic::Status>;
        async fn list_process_children(
            &self,
            request: tonic::Request<super::ListProcessChildrenRequest>,
        ) -> Result<tonic::Response<super::ProcessChildrenReply>, tonic::Status>;
        async fn list_process_log_entries(
            &self,
            request: tonic::Request<super::ProcessLogRequest>,
        ) -> Result<tonic::Response<super::ProcessLogReply>, tonic::Status>;
        async fn list_process_streams(
            &self,
            request: tonic::Request<super::ListProcessStreamsRequest>,
        ) -> Result<tonic::Response<super::ListStreamsReply>, tonic::Status>;
        async fn list_recent_processes(
            &self,
            request: tonic::Request<super::RecentProcessesRequest>,
        ) -> Result<tonic::Response<super::ProcessListReply>, tonic::Status>;
        async fn list_stream_blocks(
            &self,
            request: tonic::Request<super::ListStreamBlocksRequest>,
        ) -> Result<tonic::Response<super::ListStreamBlocksReply>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct PerformanceAnalyticsServer<T: PerformanceAnalytics> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: PerformanceAnalytics> PerformanceAnalyticsServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for PerformanceAnalyticsServer<T>
    where
        T: PerformanceAnalytics,
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
                "/analytics.PerformanceAnalytics/block_spans" => {
                    #[allow(non_camel_case_types)]
                    struct block_spansSvc<T: PerformanceAnalytics>(pub Arc<T>);
                    impl<T: PerformanceAnalytics>
                        tonic::server::UnaryService<super::BlockSpansRequest>
                        for block_spansSvc<T>
                    {
                        type Response = super::BlockSpansReply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BlockSpansRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).block_spans(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = block_spansSvc(inner);
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
                "/analytics.PerformanceAnalytics/process_cumulative_call_graph" => {
                    #[allow(non_camel_case_types)]
                    struct process_cumulative_call_graphSvc<T: PerformanceAnalytics>(pub Arc<T>);
                    impl<T: PerformanceAnalytics>
                        tonic::server::UnaryService<super::ProcessCumulativeCallGraphRequest>
                        for process_cumulative_call_graphSvc<T>
                    {
                        type Response = super::CumulativeCallGraphReply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ProcessCumulativeCallGraphRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).process_cumulative_call_graph(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = process_cumulative_call_graphSvc(inner);
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
                "/analytics.PerformanceAnalytics/find_process" => {
                    #[allow(non_camel_case_types)]
                    struct find_processSvc<T: PerformanceAnalytics>(pub Arc<T>);
                    impl<T: PerformanceAnalytics>
                        tonic::server::UnaryService<super::FindProcessRequest>
                        for find_processSvc<T>
                    {
                        type Response = super::FindProcessReply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FindProcessRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).find_process(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = find_processSvc(inner);
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
                "/analytics.PerformanceAnalytics/list_process_children" => {
                    #[allow(non_camel_case_types)]
                    struct list_process_childrenSvc<T: PerformanceAnalytics>(pub Arc<T>);
                    impl<T: PerformanceAnalytics>
                        tonic::server::UnaryService<super::ListProcessChildrenRequest>
                        for list_process_childrenSvc<T>
                    {
                        type Response = super::ProcessChildrenReply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListProcessChildrenRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_process_children(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = list_process_childrenSvc(inner);
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
                "/analytics.PerformanceAnalytics/list_process_log_entries" => {
                    #[allow(non_camel_case_types)]
                    struct list_process_log_entriesSvc<T: PerformanceAnalytics>(pub Arc<T>);
                    impl<T: PerformanceAnalytics>
                        tonic::server::UnaryService<super::ProcessLogRequest>
                        for list_process_log_entriesSvc<T>
                    {
                        type Response = super::ProcessLogReply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ProcessLogRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).list_process_log_entries(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = list_process_log_entriesSvc(inner);
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
                "/analytics.PerformanceAnalytics/list_process_streams" => {
                    #[allow(non_camel_case_types)]
                    struct list_process_streamsSvc<T: PerformanceAnalytics>(pub Arc<T>);
                    impl<T: PerformanceAnalytics>
                        tonic::server::UnaryService<super::ListProcessStreamsRequest>
                        for list_process_streamsSvc<T>
                    {
                        type Response = super::ListStreamsReply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListProcessStreamsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_process_streams(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = list_process_streamsSvc(inner);
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
                "/analytics.PerformanceAnalytics/list_recent_processes" => {
                    #[allow(non_camel_case_types)]
                    struct list_recent_processesSvc<T: PerformanceAnalytics>(pub Arc<T>);
                    impl<T: PerformanceAnalytics>
                        tonic::server::UnaryService<super::RecentProcessesRequest>
                        for list_recent_processesSvc<T>
                    {
                        type Response = super::ProcessListReply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RecentProcessesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_recent_processes(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = list_recent_processesSvc(inner);
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
                "/analytics.PerformanceAnalytics/list_stream_blocks" => {
                    #[allow(non_camel_case_types)]
                    struct list_stream_blocksSvc<T: PerformanceAnalytics>(pub Arc<T>);
                    impl<T: PerformanceAnalytics>
                        tonic::server::UnaryService<super::ListStreamBlocksRequest>
                        for list_stream_blocksSvc<T>
                    {
                        type Response = super::ListStreamBlocksReply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListStreamBlocksRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_stream_blocks(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = list_stream_blocksSvc(inner);
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
    impl<T: PerformanceAnalytics> Clone for PerformanceAnalyticsServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: PerformanceAnalytics> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: PerformanceAnalytics> tonic::transport::NamedService for PerformanceAnalyticsServer<T> {
        const NAME: &'static str = "analytics.PerformanceAnalytics";
    }
}
