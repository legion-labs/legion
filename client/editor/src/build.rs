#[cfg(feature = "custom-protocol")]
fn build_web_app() {
  use std::process::Command;
  use which::which;

  let yarn_path = which("yarn").unwrap();

  Command::new(yarn_path)
    .arg("generate")
    .current_dir("frontend")
    .spawn()
    .unwrap();
}

#[cfg(not(feature = "custom-protocol"))]
fn fake_build_web_app() {
  use std::fs::create_dir_all;

  create_dir_all("frontend/dist");
}

fn main() {
  println!("cargo:rerun-if-changed=frontend/dist");

  #[cfg(feature = "custom-protocol")]
  build_web_app();
  #[cfg(not(feature = "custom-protocol"))]
  fake_build_web_app();

  tauri_build::build()
}
