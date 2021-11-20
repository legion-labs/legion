use std::path::{Path, PathBuf};

pub fn compile_protos(proto: impl AsRef<Path>) -> std::io::Result<()> {
    let proto_path: &Path = proto.as_ref();

    let proto_dir = proto_path
        .parent()
        .expect("proto file should reside in a directory");

    println!("cargo:rerun-if-changed={}", proto_path.display());

    let out_dir = PathBuf::from("cgen").join("proto");
    std::fs::create_dir_all(&out_dir)?;

    tonic_build::configure()
        .out_dir(&out_dir)
        .compile(&[proto_path], &[proto_dir])?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "run_cgen")]
    compile_protos("./editor.proto")?;

    Ok(())
}
