use std::path::Path;

// TODO: Put this in lgn_graphics_cgen or in it's own lib
fn build_graphics_cgen(root_file: &impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    // build context
    let out_dir = Path::new(&std::env::var("OUT_DIR").unwrap()).join("codegen");
    let mut ctx_builder = lgn_graphics_cgen::run::CGenContextBuilder::new();
    ctx_builder.set_root_file(root_file).unwrap();
    ctx_builder.set_out_dir(&out_dir).unwrap();
    ctx_builder.set_crate_name(std::env::var("CARGO_PKG_NAME").unwrap());

    // run generation
    let result = lgn_graphics_cgen::run::run(&ctx_builder.build());
    match &result {
        Ok(build_result) => {
            for input_dependency in &build_result.input_dependencies {
                println!("cargo:rerun-if-changed={}", input_dependency.display());
            }
        }
        Err(e) => {
            for msg in e.chain() {
                eprintln!("{}", msg);
            }
        }
    }
    result.map(|_| ()).map_err(std::convert::Into::into)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // #[cfg(feature = "run-codegen")]
    let root_cgen = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("gpu")
        .join("codegen")
        .join("root.cgen");
    build_graphics_cgen(&root_cgen)?;
    Ok(())
}
