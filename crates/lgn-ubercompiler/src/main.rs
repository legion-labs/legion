// crate-specific lint exceptions:
//#![allow()]

use lgn_data_compiler::compiler_api::{multi_compiler_main, CompilerError};

fn main() -> Result<(), CompilerError> {
    multi_compiler_main(lgn_ubercompiler::create())
}
