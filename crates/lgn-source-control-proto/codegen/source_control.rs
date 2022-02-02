#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IndexExistsRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IndexExistsResponse {
    #[prost(bool, tag = "1")]
    pub exists: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateIndexRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateIndexResponse {
    #[prost(string, tag = "1")]
    pub blob_storage_url: ::prost::alloc::string::String,
    #[prost(bool, tag = "2")]
    pub already_exists: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DestroyIndexRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DestroyIndexResponse {
    #[prost(bool, tag = "1")]
    pub does_not_exist: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlobStorageUrlRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlobStorageUrlResponse {
    #[prost(string, tag = "1")]
    pub blob_storage_url: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisterWorkspaceRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub workspace_registration: ::core::option::Option<WorkspaceRegistration>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisterWorkspaceResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkspaceRegistration {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub owner: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FindBranchRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub branch_name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FindBranchResponse {
    #[prost(message, optional, tag = "1")]
    pub branch: ::core::option::Option<Branch>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Branch {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub head: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub parent: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub lock_domain_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadBranchesRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadBranchesResponse {
    #[prost(message, repeated, tag = "1")]
    pub branches: ::prost::alloc::vec::Vec<Branch>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FindBranchesInLockDomainRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub lock_domain_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FindBranchesInLockDomainResponse {
    #[prost(message, repeated, tag = "1")]
    pub branches: ::prost::alloc::vec::Vec<Branch>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadCommitRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub commit_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadCommitResponse {
    #[prost(message, optional, tag = "1")]
    pub commit: ::core::option::Option<Commit>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Commit {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub owner: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub message: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "4")]
    pub changes: ::prost::alloc::vec::Vec<Change>,
    #[prost(string, tag = "5")]
    pub root_tree_id: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "6")]
    pub parents: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "7")]
    pub timestamp: ::core::option::Option<::prost_types::Timestamp>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Change {
    #[prost(string, tag = "1")]
    pub canonical_path: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub change_type: ::core::option::Option<ChangeType>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChangeType {
    #[prost(string, tag = "1")]
    pub old_hash: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub new_hash: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadTreeRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub tree_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadTreeResponse {
    #[prost(message, optional, tag = "1")]
    pub tree: ::core::option::Option<Tree>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Tree {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub hash: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub children: ::prost::alloc::vec::Vec<Tree>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InsertLockRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub lock: ::core::option::Option<Lock>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InsertLockResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Lock {
    #[prost(string, tag = "1")]
    pub relative_path: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub lock_domain_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub workspace_id: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub branch_name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FindLockRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub lock_domain_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub canonical_relative_path: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FindLockResponse {
    #[prost(message, optional, tag = "1")]
    pub lock: ::core::option::Option<Lock>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FindLocksInDomainRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub lock_domain_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FindLocksInDomainResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<Lock>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SaveTreeRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub tree: ::core::option::Option<Tree>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SaveTreeResponse {
    #[prost(string, tag = "1")]
    pub tree_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InsertCommitRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub commit: ::core::option::Option<Commit>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InsertCommitResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitToBranchRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub commit: ::core::option::Option<Commit>,
    #[prost(message, optional, tag = "3")]
    pub branch: ::core::option::Option<Branch>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitToBranchResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitExistsRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub commit_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitExistsResponse {
    #[prost(bool, tag = "1")]
    pub exists: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateBranchRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub branch: ::core::option::Option<Branch>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateBranchResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InsertBranchRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub branch: ::core::option::Option<Branch>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InsertBranchResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClearLockRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub lock_domain_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub canonical_relative_path: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClearLockResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CountLocksInDomainRequest {
    #[prost(string, tag = "1")]
    pub repository_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub lock_domain_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CountLocksInDomainResponse {
    #[prost(int32, tag = "1")]
    pub count: i32,
}
#[doc = r" Generated client implementations."]
pub mod source_control_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct SourceControlClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl SourceControlClient<tonic::transport::Channel> {
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
    impl<T> SourceControlClient<T>
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
        ) -> SourceControlClient<InterceptedService<T, F>>
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
            SourceControlClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn index_exists(
            &mut self,
            request: impl tonic::IntoRequest<super::IndexExistsRequest>,
        ) -> Result<tonic::Response<super::IndexExistsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/source_control.SourceControl/IndexExists");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_index(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateIndexRequest>,
        ) -> Result<tonic::Response<super::CreateIndexResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/source_control.SourceControl/CreateIndex");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn destroy_index(
            &mut self,
            request: impl tonic::IntoRequest<super::DestroyIndexRequest>,
        ) -> Result<tonic::Response<super::DestroyIndexResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/source_control.SourceControl/DestroyIndex");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_blob_storage_url(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBlobStorageUrlRequest>,
        ) -> Result<tonic::Response<super::GetBlobStorageUrlResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/source_control.SourceControl/GetBlobStorageUrl",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn register_workspace(
            &mut self,
            request: impl tonic::IntoRequest<super::RegisterWorkspaceRequest>,
        ) -> Result<tonic::Response<super::RegisterWorkspaceResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/source_control.SourceControl/RegisterWorkspace",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn find_branch(
            &mut self,
            request: impl tonic::IntoRequest<super::FindBranchRequest>,
        ) -> Result<tonic::Response<super::FindBranchResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/source_control.SourceControl/FindBranch");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn read_branches(
            &mut self,
            request: impl tonic::IntoRequest<super::ReadBranchesRequest>,
        ) -> Result<tonic::Response<super::ReadBranchesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/source_control.SourceControl/ReadBranches");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn find_branches_in_lock_domain(
            &mut self,
            request: impl tonic::IntoRequest<super::FindBranchesInLockDomainRequest>,
        ) -> Result<tonic::Response<super::FindBranchesInLockDomainResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/source_control.SourceControl/FindBranchesInLockDomain",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn read_commit(
            &mut self,
            request: impl tonic::IntoRequest<super::ReadCommitRequest>,
        ) -> Result<tonic::Response<super::ReadCommitResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/source_control.SourceControl/ReadCommit");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn read_tree(
            &mut self,
            request: impl tonic::IntoRequest<super::ReadTreeRequest>,
        ) -> Result<tonic::Response<super::ReadTreeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/source_control.SourceControl/ReadTree");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn insert_lock(
            &mut self,
            request: impl tonic::IntoRequest<super::InsertLockRequest>,
        ) -> Result<tonic::Response<super::InsertLockResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/source_control.SourceControl/InsertLock");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn find_lock(
            &mut self,
            request: impl tonic::IntoRequest<super::FindLockRequest>,
        ) -> Result<tonic::Response<super::FindLockResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/source_control.SourceControl/FindLock");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn find_locks_in_domain(
            &mut self,
            request: impl tonic::IntoRequest<super::FindLocksInDomainRequest>,
        ) -> Result<tonic::Response<super::FindLocksInDomainResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/source_control.SourceControl/FindLocksInDomain",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn save_tree(
            &mut self,
            request: impl tonic::IntoRequest<super::SaveTreeRequest>,
        ) -> Result<tonic::Response<super::SaveTreeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/source_control.SourceControl/SaveTree");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn insert_commit(
            &mut self,
            request: impl tonic::IntoRequest<super::InsertCommitRequest>,
        ) -> Result<tonic::Response<super::InsertCommitResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/source_control.SourceControl/InsertCommit");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn commit_to_branch(
            &mut self,
            request: impl tonic::IntoRequest<super::CommitToBranchRequest>,
        ) -> Result<tonic::Response<super::CommitToBranchResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/source_control.SourceControl/CommitToBranch",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn commit_exists(
            &mut self,
            request: impl tonic::IntoRequest<super::CommitExistsRequest>,
        ) -> Result<tonic::Response<super::CommitExistsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/source_control.SourceControl/CommitExists");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_branch(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateBranchRequest>,
        ) -> Result<tonic::Response<super::UpdateBranchResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/source_control.SourceControl/UpdateBranch");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn insert_branch(
            &mut self,
            request: impl tonic::IntoRequest<super::InsertBranchRequest>,
        ) -> Result<tonic::Response<super::InsertBranchResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/source_control.SourceControl/InsertBranch");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn clear_lock(
            &mut self,
            request: impl tonic::IntoRequest<super::ClearLockRequest>,
        ) -> Result<tonic::Response<super::ClearLockResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/source_control.SourceControl/ClearLock");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn count_locks_in_domain(
            &mut self,
            request: impl tonic::IntoRequest<super::CountLocksInDomainRequest>,
        ) -> Result<tonic::Response<super::CountLocksInDomainResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/source_control.SourceControl/CountLocksInDomain",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod source_control_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with SourceControlServer."]
    #[async_trait]
    pub trait SourceControl: Send + Sync + 'static {
        async fn index_exists(
            &self,
            request: tonic::Request<super::IndexExistsRequest>,
        ) -> Result<tonic::Response<super::IndexExistsResponse>, tonic::Status>;
        async fn create_index(
            &self,
            request: tonic::Request<super::CreateIndexRequest>,
        ) -> Result<tonic::Response<super::CreateIndexResponse>, tonic::Status>;
        async fn destroy_index(
            &self,
            request: tonic::Request<super::DestroyIndexRequest>,
        ) -> Result<tonic::Response<super::DestroyIndexResponse>, tonic::Status>;
        async fn get_blob_storage_url(
            &self,
            request: tonic::Request<super::GetBlobStorageUrlRequest>,
        ) -> Result<tonic::Response<super::GetBlobStorageUrlResponse>, tonic::Status>;
        async fn register_workspace(
            &self,
            request: tonic::Request<super::RegisterWorkspaceRequest>,
        ) -> Result<tonic::Response<super::RegisterWorkspaceResponse>, tonic::Status>;
        async fn find_branch(
            &self,
            request: tonic::Request<super::FindBranchRequest>,
        ) -> Result<tonic::Response<super::FindBranchResponse>, tonic::Status>;
        async fn read_branches(
            &self,
            request: tonic::Request<super::ReadBranchesRequest>,
        ) -> Result<tonic::Response<super::ReadBranchesResponse>, tonic::Status>;
        async fn find_branches_in_lock_domain(
            &self,
            request: tonic::Request<super::FindBranchesInLockDomainRequest>,
        ) -> Result<tonic::Response<super::FindBranchesInLockDomainResponse>, tonic::Status>;
        async fn read_commit(
            &self,
            request: tonic::Request<super::ReadCommitRequest>,
        ) -> Result<tonic::Response<super::ReadCommitResponse>, tonic::Status>;
        async fn read_tree(
            &self,
            request: tonic::Request<super::ReadTreeRequest>,
        ) -> Result<tonic::Response<super::ReadTreeResponse>, tonic::Status>;
        async fn insert_lock(
            &self,
            request: tonic::Request<super::InsertLockRequest>,
        ) -> Result<tonic::Response<super::InsertLockResponse>, tonic::Status>;
        async fn find_lock(
            &self,
            request: tonic::Request<super::FindLockRequest>,
        ) -> Result<tonic::Response<super::FindLockResponse>, tonic::Status>;
        async fn find_locks_in_domain(
            &self,
            request: tonic::Request<super::FindLocksInDomainRequest>,
        ) -> Result<tonic::Response<super::FindLocksInDomainResponse>, tonic::Status>;
        async fn save_tree(
            &self,
            request: tonic::Request<super::SaveTreeRequest>,
        ) -> Result<tonic::Response<super::SaveTreeResponse>, tonic::Status>;
        async fn insert_commit(
            &self,
            request: tonic::Request<super::InsertCommitRequest>,
        ) -> Result<tonic::Response<super::InsertCommitResponse>, tonic::Status>;
        async fn commit_to_branch(
            &self,
            request: tonic::Request<super::CommitToBranchRequest>,
        ) -> Result<tonic::Response<super::CommitToBranchResponse>, tonic::Status>;
        async fn commit_exists(
            &self,
            request: tonic::Request<super::CommitExistsRequest>,
        ) -> Result<tonic::Response<super::CommitExistsResponse>, tonic::Status>;
        async fn update_branch(
            &self,
            request: tonic::Request<super::UpdateBranchRequest>,
        ) -> Result<tonic::Response<super::UpdateBranchResponse>, tonic::Status>;
        async fn insert_branch(
            &self,
            request: tonic::Request<super::InsertBranchRequest>,
        ) -> Result<tonic::Response<super::InsertBranchResponse>, tonic::Status>;
        async fn clear_lock(
            &self,
            request: tonic::Request<super::ClearLockRequest>,
        ) -> Result<tonic::Response<super::ClearLockResponse>, tonic::Status>;
        async fn count_locks_in_domain(
            &self,
            request: tonic::Request<super::CountLocksInDomainRequest>,
        ) -> Result<tonic::Response<super::CountLocksInDomainResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct SourceControlServer<T: SourceControl> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: SourceControl> SourceControlServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for SourceControlServer<T>
    where
        T: SourceControl,
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
                "/source_control.SourceControl/IndexExists" => {
                    #[allow(non_camel_case_types)]
                    struct IndexExistsSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl> tonic::server::UnaryService<super::IndexExistsRequest>
                        for IndexExistsSvc<T>
                    {
                        type Response = super::IndexExistsResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::IndexExistsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).index_exists(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = IndexExistsSvc(inner);
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
                "/source_control.SourceControl/CreateIndex" => {
                    #[allow(non_camel_case_types)]
                    struct CreateIndexSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl> tonic::server::UnaryService<super::CreateIndexRequest>
                        for CreateIndexSvc<T>
                    {
                        type Response = super::CreateIndexResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateIndexRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_index(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateIndexSvc(inner);
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
                "/source_control.SourceControl/DestroyIndex" => {
                    #[allow(non_camel_case_types)]
                    struct DestroyIndexSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl> tonic::server::UnaryService<super::DestroyIndexRequest>
                        for DestroyIndexSvc<T>
                    {
                        type Response = super::DestroyIndexResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DestroyIndexRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).destroy_index(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DestroyIndexSvc(inner);
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
                "/source_control.SourceControl/GetBlobStorageUrl" => {
                    #[allow(non_camel_case_types)]
                    struct GetBlobStorageUrlSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl>
                        tonic::server::UnaryService<super::GetBlobStorageUrlRequest>
                        for GetBlobStorageUrlSvc<T>
                    {
                        type Response = super::GetBlobStorageUrlResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetBlobStorageUrlRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_blob_storage_url(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetBlobStorageUrlSvc(inner);
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
                "/source_control.SourceControl/RegisterWorkspace" => {
                    #[allow(non_camel_case_types)]
                    struct RegisterWorkspaceSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl>
                        tonic::server::UnaryService<super::RegisterWorkspaceRequest>
                        for RegisterWorkspaceSvc<T>
                    {
                        type Response = super::RegisterWorkspaceResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RegisterWorkspaceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).register_workspace(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RegisterWorkspaceSvc(inner);
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
                "/source_control.SourceControl/FindBranch" => {
                    #[allow(non_camel_case_types)]
                    struct FindBranchSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl> tonic::server::UnaryService<super::FindBranchRequest> for FindBranchSvc<T> {
                        type Response = super::FindBranchResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FindBranchRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).find_branch(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = FindBranchSvc(inner);
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
                "/source_control.SourceControl/ReadBranches" => {
                    #[allow(non_camel_case_types)]
                    struct ReadBranchesSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl> tonic::server::UnaryService<super::ReadBranchesRequest>
                        for ReadBranchesSvc<T>
                    {
                        type Response = super::ReadBranchesResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ReadBranchesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).read_branches(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ReadBranchesSvc(inner);
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
                "/source_control.SourceControl/FindBranchesInLockDomain" => {
                    #[allow(non_camel_case_types)]
                    struct FindBranchesInLockDomainSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl>
                        tonic::server::UnaryService<super::FindBranchesInLockDomainRequest>
                        for FindBranchesInLockDomainSvc<T>
                    {
                        type Response = super::FindBranchesInLockDomainResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FindBranchesInLockDomainRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).find_branches_in_lock_domain(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = FindBranchesInLockDomainSvc(inner);
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
                "/source_control.SourceControl/ReadCommit" => {
                    #[allow(non_camel_case_types)]
                    struct ReadCommitSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl> tonic::server::UnaryService<super::ReadCommitRequest> for ReadCommitSvc<T> {
                        type Response = super::ReadCommitResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ReadCommitRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).read_commit(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ReadCommitSvc(inner);
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
                "/source_control.SourceControl/ReadTree" => {
                    #[allow(non_camel_case_types)]
                    struct ReadTreeSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl> tonic::server::UnaryService<super::ReadTreeRequest> for ReadTreeSvc<T> {
                        type Response = super::ReadTreeResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ReadTreeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).read_tree(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ReadTreeSvc(inner);
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
                "/source_control.SourceControl/InsertLock" => {
                    #[allow(non_camel_case_types)]
                    struct InsertLockSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl> tonic::server::UnaryService<super::InsertLockRequest> for InsertLockSvc<T> {
                        type Response = super::InsertLockResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::InsertLockRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).insert_lock(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = InsertLockSvc(inner);
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
                "/source_control.SourceControl/FindLock" => {
                    #[allow(non_camel_case_types)]
                    struct FindLockSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl> tonic::server::UnaryService<super::FindLockRequest> for FindLockSvc<T> {
                        type Response = super::FindLockResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FindLockRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).find_lock(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = FindLockSvc(inner);
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
                "/source_control.SourceControl/FindLocksInDomain" => {
                    #[allow(non_camel_case_types)]
                    struct FindLocksInDomainSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl>
                        tonic::server::UnaryService<super::FindLocksInDomainRequest>
                        for FindLocksInDomainSvc<T>
                    {
                        type Response = super::FindLocksInDomainResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FindLocksInDomainRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).find_locks_in_domain(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = FindLocksInDomainSvc(inner);
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
                "/source_control.SourceControl/SaveTree" => {
                    #[allow(non_camel_case_types)]
                    struct SaveTreeSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl> tonic::server::UnaryService<super::SaveTreeRequest> for SaveTreeSvc<T> {
                        type Response = super::SaveTreeResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SaveTreeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).save_tree(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SaveTreeSvc(inner);
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
                "/source_control.SourceControl/InsertCommit" => {
                    #[allow(non_camel_case_types)]
                    struct InsertCommitSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl> tonic::server::UnaryService<super::InsertCommitRequest>
                        for InsertCommitSvc<T>
                    {
                        type Response = super::InsertCommitResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::InsertCommitRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).insert_commit(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = InsertCommitSvc(inner);
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
                "/source_control.SourceControl/CommitToBranch" => {
                    #[allow(non_camel_case_types)]
                    struct CommitToBranchSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl> tonic::server::UnaryService<super::CommitToBranchRequest>
                        for CommitToBranchSvc<T>
                    {
                        type Response = super::CommitToBranchResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CommitToBranchRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).commit_to_branch(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CommitToBranchSvc(inner);
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
                "/source_control.SourceControl/CommitExists" => {
                    #[allow(non_camel_case_types)]
                    struct CommitExistsSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl> tonic::server::UnaryService<super::CommitExistsRequest>
                        for CommitExistsSvc<T>
                    {
                        type Response = super::CommitExistsResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CommitExistsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).commit_exists(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CommitExistsSvc(inner);
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
                "/source_control.SourceControl/UpdateBranch" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateBranchSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl> tonic::server::UnaryService<super::UpdateBranchRequest>
                        for UpdateBranchSvc<T>
                    {
                        type Response = super::UpdateBranchResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateBranchRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).update_branch(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateBranchSvc(inner);
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
                "/source_control.SourceControl/InsertBranch" => {
                    #[allow(non_camel_case_types)]
                    struct InsertBranchSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl> tonic::server::UnaryService<super::InsertBranchRequest>
                        for InsertBranchSvc<T>
                    {
                        type Response = super::InsertBranchResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::InsertBranchRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).insert_branch(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = InsertBranchSvc(inner);
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
                "/source_control.SourceControl/ClearLock" => {
                    #[allow(non_camel_case_types)]
                    struct ClearLockSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl> tonic::server::UnaryService<super::ClearLockRequest> for ClearLockSvc<T> {
                        type Response = super::ClearLockResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ClearLockRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).clear_lock(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ClearLockSvc(inner);
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
                "/source_control.SourceControl/CountLocksInDomain" => {
                    #[allow(non_camel_case_types)]
                    struct CountLocksInDomainSvc<T: SourceControl>(pub Arc<T>);
                    impl<T: SourceControl>
                        tonic::server::UnaryService<super::CountLocksInDomainRequest>
                        for CountLocksInDomainSvc<T>
                    {
                        type Response = super::CountLocksInDomainResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CountLocksInDomainRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).count_locks_in_domain(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CountLocksInDomainSvc(inner);
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
    impl<T: SourceControl> Clone for SourceControlServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: SourceControl> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: SourceControl> tonic::transport::NamedService for SourceControlServer<T> {
        const NAME: &'static str = "source_control.SourceControl";
    }
}
