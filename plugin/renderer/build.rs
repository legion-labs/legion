use graphics_cgen::run::CGenContextBuilder;
use std::env;

fn main() {
    // initialize context
    let crate_folder = env!("CARGO_MANIFEST_DIR").to_string();

    let mut root_cgen = crate_folder.clone();
    root_cgen.push_str("/src/root.cgen");

    let mut outdir_hlsl = crate_folder.clone();
    outdir_hlsl.push_str("/cgen/hlsl/");

    let mut outdir_rust = crate_folder.clone();
    outdir_rust.push_str("/cgen/rust/");

    let mut ctx_builder = CGenContextBuilder::new();
    ctx_builder.set_root_file(&root_cgen).unwrap();
    ctx_builder.set_outdir_hlsl(&outdir_hlsl).unwrap();
    ctx_builder.set_outdir_rust(&outdir_rust).unwrap();

    // run the generation
    let ctx = ctx_builder.build();
    match graphics_cgen::run::run(&ctx) {
        Ok(_) => {
            println!("Code generation succeeded");
        }
        Err(e) => {
            for msg in e.chain() {
                eprintln!("{}", msg);
            }
        }
    }
}
