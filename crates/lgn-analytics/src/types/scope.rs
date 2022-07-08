#[derive(Clone, PartialEq, prost::Message)]
pub struct ScopeDesc {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(string, tag = "2")]
    pub filename: String,
    #[prost(uint32, tag = "3")]
    pub line: u32,
    #[prost(uint32, tag = "4")]
    pub hash: u32,
}
