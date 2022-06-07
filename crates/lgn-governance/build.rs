fn main() {
    println!("cargo:rerun-if-changed=migrations");

    let options = lgn_api_codegen::RustOptions::default();

    lgn_api_codegen::generate!(
        lgn_api_codegen::Language::Rust(options),
        "apis",
        [
            "space",
            "permission",
            "role",
            "session",
            "workspace",
            "user"
        ],
    );
}
