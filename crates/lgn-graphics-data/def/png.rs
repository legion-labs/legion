#[resource]
#[legion(offline_only)]
pub struct Png {
    #[legion(read_only)]
    pub width: u32,

    #[legion(read_only)]
    pub height: u32,

    #[legion(read_only)]
    pub layers: Vec<String>,

    pub content_id: String,
}
