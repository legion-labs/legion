#[cfg(feature = "custom-protocol")]
fn build_web_app() {
  use std::process::Command;
  use which::which;

  let yarn_path = which("yarn").unwrap();

  let mut process = Command::new(yarn_path)
    .arg("generate")
    .current_dir("frontend")
    .spawn()
    .unwrap();

  std::process::exit(process.wait().unwrap().code().unwrap());
}

fn main() {
  println!("cargo:rerun-if-changed=frontend/dist");

  #[cfg(feature = "custom-protocol")]
  build_web_app();

  tauri_build::build()
}
