use std::ffi::OsStr;
use std::process::Command;

fn run<S: AsRef<OsStr>>(command_path: S, arg: &str, dir: &str) {
    let mut process = Command::new(command_path.as_ref())
        .arg(arg)
        .current_dir(dir)
        .spawn()
        .unwrap();

    let exit_code = process.wait().unwrap().code().unwrap();

    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}

#[cfg(feature = "custom-protocol")]
fn build_web_app() {
    if let Ok(yarn_path) = which::which("yarn") {
        let frontend_dir = "frontend";

        run(&yarn_path, "install", frontend_dir);
        run(&yarn_path, "setup", frontend_dir);
        run(yarn_path, "build", frontend_dir);

        std::process::exit(0);
    } else {
        std::fs::create_dir_all("frontend/dist").unwrap();
        std::fs::write("frontend/dist/index.html", "Yarn missing from path").unwrap();
        println!("cargo:rerun-if-env-changed=PATH");
    }
}

fn main() {
    #[cfg(feature = "custom-protocol")]
    {
        // JS ecosystem forces us to have output files in our sources hiearchy
        // we are filtering files
        std::fs::read_dir("frontend")
            .unwrap()
            .map(|res| res.map(|entry| entry.path()))
            .filter_map(|path| {
                if let Ok(path) = path {
                    if let Some(file_name) = path.file_name() {
                        if file_name != "dist" && file_name != "node_modules" {
                            return Some(path);
                        }
                    }
                }

                None
            })
            .for_each(|path| {
                // to_string_lossy should be fine here, our first level folder names are clean
                println!("cargo:rerun-if-changed={}", path.to_string_lossy())
            });

        build_web_app();
    }

    tauri_build::build()
}
