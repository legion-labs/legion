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

    let exit_code = process.wait().unwrap().code().unwrap();

    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    let mut process = Command::new(yarn_path)
        .arg("build")
        .current_dir("frontend")
        .spawn()
        .unwrap();

    std::process::exit(process.wait().unwrap().code().unwrap());
}

fn main() {
    #[cfg(feature = "custom-protocol")]
    {
        // JS ecosystem forces us to have output files in our sources hiearchy
        // we are filtering files
        std::fs::read_dir("frontend")
            .unwrap()
            .map(|res| res.map(|entry| entry.path()))
            .filter(|path| {
                if let Ok(path) = path {
                    if let Some(file_name) = path.file_name() {
                        return file_name != "dist"
                            && file_name != "node_modules"
                            && file_name != ".nuxt";
                    }
                }
                false
            })
            .for_each(|path| {
                // to_string_lossy should be fine here, our first level folder names are clean
                println!("cargo:rerun-if-changed={}", path.unwrap().to_string_lossy())
            });

        build_web_app();
    }

    tauri_build::build()
}
