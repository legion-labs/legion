use crate::lsc_lib::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalChange {
    pub relative_path: String,
    pub change_type: String, //edit, add, delete
}

pub fn track_new_file(file_to_add_specified: &Path) -> Result<(), String> {
    let file_to_add_buf = make_path_absolute(file_to_add_specified);
    let file_to_add = file_to_add_buf.as_path();
    match fs::metadata(file_to_add) {
        Ok(_file_metadata) => {
            match file_to_add.parent() {
                None => {
                    return Err(format!(
                        "Error finding parent workspace of {:?}",
                        file_to_add
                    ));
                }
                Some(parent) => {
                    let workspace_root = make_path_absolute(find_workspace_root(parent)?);
                    let local_edit_id = uuid::Uuid::new_v4().to_string();
                    let local_edit_obj_path = workspace_root
                        .join(".lsc/local_edits/")
                        .join(local_edit_id + ".json");

                    //todo: lock the new file before recording the local change
                    let local_change = LocalChange {
                        relative_path: path_to_string(
                            path_relative_to(file_to_add, workspace_root.as_path())?.as_path(),
                        ),
                        change_type: String::from("add"),
                    };

                    match serde_json::to_string(&local_change) {
                        Ok(json_spec) => {
                            write_file(local_edit_obj_path.as_path(), json_spec.as_bytes())?;
                        }
                        Err(e) => {
                            return Err(format!("Error formatting local change spec: {}", e));
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!(
                "Error reading file metadata {:?}: {}",
                file_to_add, e
            ))
        }
    }
    Ok(())
}
