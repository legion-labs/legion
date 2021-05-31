use crate::*;
use std::path::Path;

pub fn revert_file_command(path: &Path) -> Result<(), String> {
    let abs_path = make_path_absolute(path);
    let workspace_root = find_workspace_root(&abs_path)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let relative_path = path_relative_to(&abs_path, workspace_root)?;
    let local_change = find_local_change(&workspace_root, &relative_path)?;
    let parent_dir = relative_path.parent().expect("no parent to path provided");
    let workspace_branch = read_current_branch(&workspace_root)?;
    let current_commit = read_commit(&workspace_spec.repository, &workspace_branch.head)?;
    let root_tree = read_tree(&workspace_spec.repository, &current_commit.root_hash)?;
    let dir_tree = fetch_tree_subdir(&workspace_spec.repository, &root_tree, &parent_dir)?;

    if local_change.change_type != "add" {
        let file_node = dir_tree.find_file_node(
            relative_path
                .file_name()
                .expect("no file name in path specified")
                .to_str()
                .expect("invalid file name"),
        )?;
        download_blob(&workspace_spec.repository, &abs_path, &file_node.hash)?;
        make_file_read_only(&abs_path, true)?;
    }
    clear_local_change(&workspace_root, &local_change)?;
    Ok(())
}
