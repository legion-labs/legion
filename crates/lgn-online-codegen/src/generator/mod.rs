use self::rust::RustGenerator;
use crate::openapi;
use std::str::FromStr;

mod rust;

pub enum Generator {
    Rust(RustGenerator),
}

impl FromStr for Generator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rust" => Ok(Generator::Rust(RustGenerator {})),
            _ => Err(format!("Unknown generator: {}", s)),
        }
    }
}

impl Generator {
    pub fn generate(&self, spec: &openapi::Spec) {
        match self {
            Generator::Rust(rust_generator) => rust_generator.generate(spec),
        }
    }
}
