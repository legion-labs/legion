use std::env;

use compiler_material::COMPILER_INFO;
use legion_data_compiler::compiler_api::compiler_main;

fn main() {
    std::process::exit(match compiler_main(env::args(), &COMPILER_INFO) {
        Ok(_) => 0,
        Err(_) => 1,
    });
}
