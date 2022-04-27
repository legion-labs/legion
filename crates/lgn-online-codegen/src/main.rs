use crate::generator::Generator;
use std::str::FromStr;
mod errors;
mod generator;
mod openapi;

fn main() {
    let spec = openapi::Spec::from_yaml_file("openapi.yaml").unwrap();
    Generator::from_str("rust").unwrap().generate(&spec);
}
