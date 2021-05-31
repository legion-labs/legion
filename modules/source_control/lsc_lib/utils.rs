use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::result::Result;

pub fn write_file(path: &Path, contents: &[u8]) -> Result<(), String> {
    match fs::File::create(path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(contents) {
                return Err(format!("Error writing {:?}: {}", path, e));
            }
        }
        Err(e) => return Err(format!("Error writing {:?}: {}", path, e)),
    }
    Ok(())
}

pub fn read_file(path: &Path) -> Result<String, String> {
    match fs::File::open(path) {
        Ok(mut f) => {
            let mut buffer = String::new();
            match f.read_to_string(&mut buffer){
                Ok(_size) => {}
                Err(e) => return Err(format!("Error reading file {:?}: {}", path, e))
            }
            Ok(buffer)
        }
        Err(e) => return Err(format!("Error opening file {:?}: {}", path, e)),
    }
}

pub fn path_to_string(p: &Path) -> String {
    String::from(p.to_str().unwrap())
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
        Err(e) => Err(format!("{:?} not relative to {:?}: {}", p, base, e)),
    }
}
