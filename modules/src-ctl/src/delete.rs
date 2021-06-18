use crate::*;
use std::fs;
use std::path::Path;

pub fn delete_file_command(path_specified: &Path) -> Result<(), String> {
    let abs_path = make_path_absolute(path_specified);
    if !abs_path.exists() {
        return Err(format!("Error: file not found {}", abs_path.display()));
    }
    let workspace_root = find_workspace_root(&abs_path)?;
    assert_not_locked(&workspace_root, &abs_path)?;

    let relative_path = make_canonical_relative_path(&workspace_root, &abs_path)?;

    match find_local_change(&workspace_root, &relative_path) {
        SearchResult::Ok(change) => {
            return Err(format!(
                "Error: {} already tracked for {}",
                change.relative_path, change.change_type
            ));
        }
        SearchResult::Err(e) => {
            return Err(format!("Error searching in local changes: {}", e));
        }
        SearchResult::None => { //all is good
        }
    }

    //todo: lock file
    let local_change = LocalChange::new(relative_path, String::from("delete"));
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
