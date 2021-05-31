use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::result::Result;

pub fn write_file(path: &Path, contents: &[u8]) -> Result<(), String> {
    match fs::File::create(path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(contents) {
                return Err(format!("Error writing {}: {}", path.display(), e));
            }
        }
        Err(e) => return Err(format!("Error writing {}: {}", path.display(), e)),
    }
    Ok(())
}

pub fn read_text_file(path: &Path) -> Result<String, String> {
    match fs::File::open(path) {
        Ok(mut f) => {
            let mut buffer = String::new();
            match f.read_to_string(&mut buffer) {
                Ok(_size) => {}
                Err(e) => return Err(format!("Error reading file {}: {}", path.display(), e)),
            }
            Ok(buffer)
        }
        Err(e) => return Err(format!("Error opening file {}: {}", path.display(), e)),
    }
}

pub fn read_bin_file(path: &Path) -> Result<Vec<u8>, String> {
    match fs::read(path) {
        Ok(buffer) => Ok(buffer),
        Err(e) => Err(format!("Error reading file {}: {}", path.display(), e)),
    }
}

pub fn path_to_string(p: &Path) -> String {
    String::from(p.to_str().unwrap())
}

pub fn format_path(p: &Path) -> String {
    p.to_string_lossy().replace("/", "\\")
}

pub fn make_path_absolute(p: &Path) -> PathBuf {
    //fs::canonicalize is a trap - it generates crazy unusable "extended length" paths
    if p.is_absolute() {
        PathBuf::from(path_clean::clean(p.to_str().unwrap()))
    } else {
        PathBuf::from(path_clean::clean(
            std::env::current_dir().unwrap().join(p).to_str().unwrap(),
        ))
    }
}

pub fn path_relative_to(p: &Path, base: &Path) -> Result<PathBuf, String> {
    match p.strip_prefix(base) {
        Ok(res) => Ok(res.to_path_buf()),
        Err(e) => Err(format!("{} not relative to {}: {}", p.display(), base.display(), e)),
    }
}

pub fn make_file_read_only(file_path: &Path, readonly: bool) -> Result<(), String> {
    match fs::metadata(&file_path) {
        Ok(meta) => {
            let mut permissions = meta.permissions();
            permissions.set_readonly(readonly);
            if let Err(e) = fs::set_permissions(&file_path, permissions) {
                return Err(format!(
                    "Error changing file permissions for {}: {}",
                    file_path.display(),
                    e
                ));
            }
        }
        Err(e) => {
            return Err(format!(
                "Error reading file metadata for {}: {}",
                file_path.display(),
                e
            ));
        }
    }
    Ok(())
}

pub fn lz4_compress_to_file(file_path: &Path, contents: &[u8]) -> Result<(), String> {
    match std::fs::File::create(file_path) {
        Err(e) => {
            return Err(format!("Error creating file {}: {}", file_path.display(), e));
        }
        Ok(output_file) => match lz4::EncoderBuilder::new().level(10).build(output_file) {
            Err(e) => return Err(format!("Error building lz4 encoder: {}", e)),
            Ok(mut encoder) => {
                if let Err(e) = encoder.write(contents) {
                    return Err(format!("Error writing to lz4 encoder: {}", e));
                }
                if let (_w, Err(e)) = encoder.finish() {
                    return Err(format!("Error closing lz4 encoder: {}", e));
                }
                Ok(())
            }
        },
    }
}

pub fn lz4_read(compressed: &Path) -> Result<String, String> {
    match std::fs::File::open(compressed) {
        Ok(input_file) => match lz4::Decoder::new(input_file) {
            Ok(mut decoder) => {
                let mut res = String::new();
                match decoder.read_to_string(&mut res) {
                    Ok(_) => Ok(res),
                    Err(e) => {
                        return Err(format!(
                            "Error reading lz4 file {}: {}",
                            compressed.display(),
                            e
                        ));
                    }
                }
            }
            Err(e) => {
                return Err(format!(
                    "Error reading lz4 file {}: {}",
                    compressed.display(),
                    e
                ));
            }
        },
        Err(e) => {
            return Err(format!(
                "Error opening file {}: {}",
                compressed.display(),
                e
            ));
        }
    }
}

pub fn lz4_decompress(compressed: &Path, destination: &Path) -> Result<(), String> {
    match std::fs::File::open(compressed) {
        Ok(input_file) => match lz4::Decoder::new(input_file) {
            Ok(mut decoder) => match std::fs::File::create(destination) {
                Ok(mut output_file) => {
                    if let Err(e) = std::io::copy(&mut decoder, &mut output_file) {
                        return Err(format!(
                            "Error decoding from {} to {}: {}",
                            compressed.display(),
                            destination.display(),
                            e
                        ));
                    }
                }
                Err(e) => {
                    return Err(format!(
                        "Error creating file {}: {}",
                        destination.display(),
                        e
                    ));
                }
            },
            Err(e) => {
                return Err(format!(
                    "Error creating lz4 decoder from {}: {}",
                    compressed.display(),
                    e
                ));
            }
        },
        Err(e) => {
            return Err(format!(
                "Error opening file {}: {}",
                compressed.display(),
                e
            ));
        }
    }
    Ok(())
}
