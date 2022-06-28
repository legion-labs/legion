#[resource]
#[legion(offline_only)]
pub struct MultiTextResource {
    pub text_list: Vec<String>,
}
