// crate-specific lint exceptions:
//#![allow()]

use std::env;

use lgn_compiler_tex2bin::COMPILER_INFO;
use lgn_data_compiler::compiler_api::{compiler_main, CompilerError};

#[tokio::main]
async fn main() -> Result<(), CompilerError> {
    compiler_main(&env::args(), &COMPILER_INFO).await
}
