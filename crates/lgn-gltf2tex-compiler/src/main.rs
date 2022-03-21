use std::env;

use lgn_compiler_gltf2tex::COMPILER_INFO;
use lgn_data_compiler::compiler_api::{compiler_main, CompilerError};

#[tokio::main]
async fn main() -> Result<(), CompilerError> {
    compiler_main(&env::args(), &COMPILER_INFO).await
}
