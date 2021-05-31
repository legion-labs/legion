use crate::*;
use std::path::Path;

fn reference_version_name_as_commit_id(
    repo: &Path,
    workspace_root: &Path,
    reference_version_name: &str,
) -> Result<String, String> {
    match reference_version_name {
        "base" => {
            let workspace_branch = read_current_branch(&workspace_root)?;
            Ok(workspace_branch.head)
        }
        "latest" => {
            let workspace_branch = read_current_branch(&workspace_root)?;
            let branch = read_branch_from_repo(&repo, &workspace_branch.name)?;
            Ok(branch.head)
        }
        _ => {
            Ok(String::from(reference_version_name))
        }
    }
}

pub fn diff_file_command(path: &Path, reference_version_name: &str) -> Result<(), String> {
    let abs_path = make_path_absolute(path);
    let workspace_root = find_workspace_root(&abs_path)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let relative_path = path_relative_to(&abs_path, workspace_root)?;
    let ref_commit_id = reference_version_name_as_commit_id(
        &workspace_spec.repository,
        &workspace_root,
        reference_version_name,
    )?;
    let ref_file_hash = find_file_hash_at_commit(&workspace_spec.repository, &relative_path, &ref_commit_id)?;
    let base_version_contents = read_blob(&workspace_spec.repository, &ref_file_hash)?;
    let local_version_contents = read_text_file(&path)?;
    let patch = diffy::create_patch(&base_version_contents, &local_version_contents);
    println!("{}", patch);
    Ok(())
}
