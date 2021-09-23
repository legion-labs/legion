fn build_web_app() {
  use std::process::Command;
  use which::which;

  let yarn_path = which("yarn").unwrap();

  Command::new(yarn_path)
    .arg("build")
    .current_dir("frontend")
    .spawn()
    .unwrap();
}

fn main() {
  #[cfg(feature = "custom-protocol")]
  build_web_app();

  tauri_build::build()
}
