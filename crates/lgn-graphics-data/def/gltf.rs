#[resource]
#[legion(offline_only)]
pub struct Gltf {
    #[legion(read_only)]
    pub entities: Vec<String>,

    #[legion(read_only)]
    pub material: Vec<String>,

    #[legion(read_only)]
    pub textures: Vec<String>,

    #[legion(read_only)]
    pub content_id: String,
}
