#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Process {
    #[prost(string, tag = "1")]
    pub process_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub exe: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub username: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub realname: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub computer: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub distro: ::prost::alloc::string::String,
    #[prost(string, tag = "7")]
    pub cpu_brand: ::prost::alloc::string::String,
    #[prost(uint64, tag = "8")]
    pub tsc_frequency: u64,
    /// RFC 3339
    #[prost(string, tag = "9")]
    pub start_time: ::prost::alloc::string::String,
    #[prost(int64, tag = "10")]
    pub start_ticks: i64,
    #[prost(string, tag = "11")]
    pub parent_process_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UdtMember {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub type_name: ::prost::alloc::string::String,
    #[prost(uint32, tag = "3")]
    pub offset: u32,
    #[prost(uint32, tag = "4")]
    pub size: u32,
    #[prost(bool, tag = "5")]
    pub is_reference: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserDefinedType {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(uint32, tag = "2")]
    pub size: u32,
    #[prost(message, repeated, tag = "3")]
    pub members: ::prost::alloc::vec::Vec<UdtMember>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ContainerMetadata {
    #[prost(message, repeated, tag = "1")]
    pub types: ::prost::alloc::vec::Vec<UserDefinedType>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Stream {
    #[prost(string, tag = "1")]
    pub stream_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub process_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub dependencies_metadata: ::core::option::Option<ContainerMetadata>,
    #[prost(message, optional, tag = "4")]
    pub objects_metadata: ::core::option::Option<ContainerMetadata>,
    #[prost(string, repeated, tag = "5")]
    pub tags: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(map = "string, string", tag = "6")]
    pub properties:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockPayload {
    #[prost(bytes = "vec", tag = "1")]
    pub dependencies: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub objects: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Block {
    #[prost(string, tag = "1")]
    pub block_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub stream_id: ::prost::alloc::string::String,
    /// we send both RFC3339 times and ticks to be able to calibrate the tick
    /// frequency
    #[prost(string, tag = "3")]
    pub begin_time: ::prost::alloc::string::String,
    #[prost(int64, tag = "4")]
    pub begin_ticks: i64,
    #[prost(string, tag = "5")]
    pub end_time: ::prost::alloc::string::String,
    #[prost(int64, tag = "6")]
    pub end_ticks: i64,
    #[prost(message, optional, tag = "7")]
    pub payload: ::core::option::Option<BlockPayload>,
    #[prost(int32, tag = "8")]
    pub nb_objects: i32,
}
