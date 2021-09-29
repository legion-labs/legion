#[cfg(feature = "custom-protocol")]
fn build_web_app() {
    use std::process::Command;
    use which::which;

    if let Ok(yarn_path) = which("yarn") {
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
    } else {
        std::fs::create_dir_all("frontend/dist").unwrap();
        std::fs::write("frontend/dist/index.html", "Yarn missing from path").unwrap();
    }
}

fn main() {
    #[cfg(feature = "custom-protocol")]
    {
        println!("cargo:rerun-if-changed=frontend/dist");
        build_web_app();
    }

    tauri_build::build()
}
