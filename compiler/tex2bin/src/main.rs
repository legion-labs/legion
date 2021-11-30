use std::env;

use compiler_tex2bin::COMPILER_INFO;
use legion_data_compiler::compiler_api::{compiler_main, CompilerError};

fn main() -> Result<(), CompilerError> {
    compiler_main(env::args(), &COMPILER_INFO)
}
