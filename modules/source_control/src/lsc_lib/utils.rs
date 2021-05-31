use std::fs;
use std::path::Path;
use std::result::Result;
use std::io::prelude::*;

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

pub fn path_to_string(p: &Path) -> String {
    String::from(p.to_str().unwrap())
}
