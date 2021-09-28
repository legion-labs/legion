#[cfg(feature = "custom-protocol")]
fn build_web_app() {
    use std::process::Command;
    use which::which;

    let yarn_path = which("yarn").unwrap();

    let mut process = Command::new(&yarn_path)
        .arg("install")
        .current_dir("frontend")
        .spawn()
        .unwrap();

    let ec = process.wait().unwrap().code().unwrap();

    if ec != 0 {
        std::process::exit(ec);
    }

    let mut process = Command::new(yarn_path)
        .arg("generate")
        .current_dir("frontend")
        .spawn()
        .unwrap();

    std::process::exit(process.wait().unwrap().code().unwrap());
}

fn main() {
    #[cfg(feature = "custom-protocol")]
    {
        println!("cargo:rerun-if-changed=frontend/dist");
        build_web_app();
    }

    tauri_build::build()
}
