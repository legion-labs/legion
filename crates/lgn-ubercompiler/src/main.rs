// crate-specific lint exceptions:
//#![allow()]

use lgn_data_compiler::compiler_api::{multi_compiler_main, CompilerError};

#[tokio::main]
async fn main() -> Result<(), CompilerError> {
    multi_compiler_main(lgn_ubercompiler::create()).await
}
