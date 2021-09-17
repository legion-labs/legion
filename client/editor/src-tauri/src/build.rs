use std::fs;

fn main() {
  // This is annoying but the folder needs to exist or the crate won't build.
  let _ = fs::create_dir("../dist");
  tauri_build::build()
}
