use std::path::Path;

use lgn_graphics_cgen::{build_graphics_cgen, buildrs_helper::run_graphics_cgen};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build_graphics_cgen!()
}
