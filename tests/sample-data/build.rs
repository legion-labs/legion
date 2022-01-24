fn symlink(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    #[cfg(windows)]
    return std::os::windows::fs::symlink_dir(src, dst);
    #[cfg(not(windows))]
    return std::os::unix::fs::symlink(src, dst);
}

fn main() {
    let out_dir = std::path::PathBuf::from(&std::env::var("OUT_DIR").unwrap());
    if std::env::var("LGN_SYMLINK_OUT_DIR").is_ok() {
        symlink(
            &out_dir,
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("out_dir"),
        )
        .unwrap_or_default(); // allow it to failed
    }

    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("def");
    lgn_data_codegen::generate_for_directory(&directory, out_dir).unwrap();
    println!("cargo:rerun-if-changed={}", directory.display());
}
