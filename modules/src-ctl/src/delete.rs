use crate::*;
use std::fs;
use std::path::Path;

pub fn delete_file_command(path_specified: &Path) -> Result<(), String> {
    let abs_path = make_path_absolute(path_specified);
    if !abs_path.exists(){
        return Err(format!("Error: file not found {}", abs_path.display()));
    }
    let workspace_root = find_workspace_root(&abs_path)?;
    assert_not_locked(&workspace_root, &abs_path)?;
    //todo: lock file
    let local_change = LocalChange::new(
        path_relative_to(&abs_path, workspace_root.as_path())?,
        String::from("delete"),
    );
    save_local_change(&workspace_root, &local_change)?;

    make_file_read_only(&abs_path, false)?;
    if let Err(e) = fs::remove_file(&abs_path) {
        return Err(format!(
            "Error deleting local file {}: {}",
            abs_path.display(),
            e
        ));
    }
    Ok(())
}
