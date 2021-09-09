use std::path::Path;

pub fn load_data(dir: impl AsRef<Path>) {
    let dir = dir.as_ref();
    println!("loading folder {}", dir.display());
    if let Ok(entries) = dir.read_dir() {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    load_data(entry.path());
                } else {
                    assert!(!file_type.is_symlink());
                }
            }
        }
    }
}
