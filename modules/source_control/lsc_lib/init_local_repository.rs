use std::fs;

pub fn init_local_repository(directory: &str) -> Result<(), String> {
    if fs::metadata(directory).is_ok() {
        return Err(format!("{} already exists", directory));
    }
    if let Err(e) = fs::create_dir_all(format!("{}/trees", directory)) {
        return Err(format!("Error creating trees directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(format!("{}/commits", directory)) {
        return Err(format!("Error creating commits directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(format!("{}/blobs", directory)) {
        return Err(format!("Error creating blobs directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(format!("{}/branches", directory)) {
        return Err(format!("Error creating branches directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(format!("{}/workspaces", directory)) {
        return Err(format!("Error creating workspaces directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(format!("{}/locks", directory)) {
        return Err(format!("Error creating locks directory: {}", e));
    }
    Ok(())
}
