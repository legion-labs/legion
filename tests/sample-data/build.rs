use std::error::Error;

use lgn_data_codegen::generate_def;

fn main() -> Result<(), Box<dyn Error>> {
    generate_def()
}
