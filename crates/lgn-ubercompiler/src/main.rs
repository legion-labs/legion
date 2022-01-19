// crate-specific lint exceptions:
//#![allow()]

use std::env;

use lgn_data_compiler::compiler_api::{multi_compiler_main, CompilerError};

fn main() -> Result<(), CompilerError> {
    multi_compiler_main(env::args(), lgn_ubercompiler::create())
}
