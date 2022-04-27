use crate::openapi;

pub struct RustGenerator {}

impl RustGenerator {
    pub fn generate(&self, spec: &openapi::Spec) {
        println!("rust generator: {:?}", spec);
    }
}
