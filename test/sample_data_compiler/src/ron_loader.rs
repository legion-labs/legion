use std::path::Path;

pub fn load_data(root_folder: impl AsRef<Path>) {
    let root_folder = root_folder.as_ref();
    if let Ok(entries) = root_folder.read_dir() {
        let mut raw_dir = entries
            .flatten()
            .filter(|e| e.file_type().unwrap().is_dir() && e.file_name() == "raw");
        if let Some(raw_dir) = raw_dir.next() {
            load_dir(raw_dir.path());
        } else {
            eprintln!(
                "did not find a 'raw' sub-directory in {}",
                root_folder.display()
            );
        }
    } else {
        eprintln!("unable to open directory {}", root_folder.display());
    }
}

fn load_dir(dir: impl AsRef<Path>) {
    let dir = dir.as_ref();
    println!("loading folder {}", dir.display());
    if let Ok(entries) = dir.read_dir() {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    load_dir(entry.path());
                } else {
                    assert!(!file_type.is_symlink());
                    load_file(entry.path());
                }
            }
        }
    }
}

fn load_file(file: impl AsRef<Path>) {
    let file = file.as_ref();
    let name = file.file_name();
    println!("processing file {:?}", name);
}
