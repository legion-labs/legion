use crate::*;
use std::path::Path;

pub fn diff_file_command(path: &Path, reference_version_name: &str) -> Result<(),String>{
    let abs_path = make_path_absolute(path);
    let workspace_root = find_workspace_root(&abs_path)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let relative_path = path_relative_to(&abs_path, workspace_root)?;
    let parent_dir = relative_path.parent().expect("no parent to path provided");
    let workspace_branch = read_current_branch(&workspace_root)?;
    let reference_root_hash;
    match reference_version_name{
        "base" => {
            let current_commit = read_commit(&workspace_spec.repository, &workspace_branch.head)?;
            reference_root_hash = current_commit.root_hash;
        }
        "latest" =>{
            let branch = read_branch_from_repo(&workspace_spec.repository, &workspace_branch.name)?;
            let latest_commit = read_commit(&workspace_spec.repository, &branch.head)?;
            reference_root_hash = latest_commit.root_hash;
        }
        _ =>{
            let specified_commit = read_commit(&workspace_spec.repository, &reference_version_name)?;
            reference_root_hash = specified_commit.root_hash;
        }
    }
    
    let root_tree = read_tree(&workspace_spec.repository, &reference_root_hash)?;
    let dir_tree = fetch_tree_subdir(&workspace_spec.repository, &root_tree, &parent_dir)?;
    let file_node = dir_tree.find_file_node(
        relative_path
            .file_name()
            .expect("no file name in path specified")
            .to_str()
            .expect("invalid file name"),
    )?;
    let base_version_contents = read_blob(&workspace_spec.repository, &file_node.hash)?;
    let local_version_contents = read_text_file(&path)?;
    let patch = diffy::create_patch(&base_version_contents, &local_version_contents);
    println!("{}", patch);
    Ok(())
}
