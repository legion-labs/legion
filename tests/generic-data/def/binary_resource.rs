#[resource]
#[legion(offline_only)]
pub struct BinaryResource {
    #[serde(with = "serde_bytes")]
    pub content: Vec<u8>,
}
