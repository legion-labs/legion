use std::env;

use compiler_test_atoi::COMPILER_INFO;
use legion_data_compiler::compiler_api::{compiler_main, CompilerError};

fn main() -> Result<(), CompilerError> {
    compiler_main(env::args(), &COMPILER_INFO)
}
