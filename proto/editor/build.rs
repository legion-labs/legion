use std::path::{Path, PathBuf};

// this would be in a shared build lib.
pub fn compile_protos(proto: impl AsRef<Path>) -> std::io::Result<()> {
    let proto_path: &Path = proto.as_ref();

    let proto_dir = proto_path
        .parent()
        .expect("proto file should reside in a directory");

    println!("cargo:rerun-if-changed={}", proto_path.display());

    let out_dir = if cfg!(feature = "run_cgen_validate") {
        PathBuf::from(std::env::var("OUT_DIR").unwrap())
    } else {
        PathBuf::from("cgen").join("proto")
    };
    std::fs::create_dir_all(&out_dir)?;

    tonic_build::configure()
        .out_dir(&out_dir)
        .compile(&[&proto_path], &[proto_dir])?;

    if cfg!(feature = "run_cgen_validate") {
        // compare and error out
        // we can also always gen in OUT_DIR and copy or compare, probably better
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "run_cgen")]
    compile_protos("./editor.proto")?;

    Ok(())
}
