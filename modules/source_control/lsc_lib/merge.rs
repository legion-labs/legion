use crate::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;

#[derive(Serialize, Deserialize, Debug)]
pub struct MergePending {
    pub id: String,
    pub relative_path: PathBuf,
    pub base_commit_id: String,
    pub theirs_commit_id: String,
}

impl MergePending {
    pub fn new(
        relative_path: PathBuf,
        base_commit_id: String,
        theirs_commit_id: String,
    ) -> MergePending {
        let id = uuid::Uuid::new_v4().to_string();
        MergePending {
            id,
            relative_path,
            base_commit_id,
            theirs_commit_id,
        }
    }
}

pub fn save_merge_pending(
    workspace_root: &Path,
    merge_pending: &MergePending,
) -> Result<(), String> {
    let file_path = workspace_root.join(format!(".lsc/merge_pending/{}.json", &merge_pending.id));
    match serde_json::to_string(&merge_pending) {
        Ok(json) => {
            write_file(&file_path, json.as_bytes())?;
        }
        Err(e) => {
            return Err(format!("Error formatting merge pending: {}", e));
        }
    }
    Ok(())
}

fn find_merge_pending(workspace_root: &Path, relative_path: &Path) -> Result<MergePending, String> {
    let merges_pending_dir = workspace_root.join(".lsc/merge_pending");
    match merges_pending_dir.read_dir() {
        Ok(dir_iterator) => {
            for entry_res in dir_iterator {
                match entry_res {
                    Ok(entry) => {
                        let parsed: serde_json::Result<MergePending> =
                            serde_json::from_str(&read_text_file(&entry.path())?);
                        match parsed {
                            Ok(merge) => {
                                if merge.relative_path == relative_path {
                                    return Ok(merge);
                                }
                            }
                            Err(e) => {
                                return Err(format!("Error parsing {:?}: {}", entry.path(), e));
                            }
                        }
                    }
                    Err(e) => return Err(format!("Error reading pending merge entry: {}", e)),
                }
            }
        }
        Err(e) => {
            return Err(format!(
                "Error reading directory {:?}: {}",
                merges_pending_dir, e
            ))
        }
    }
    Err(format!(
        "local change {} not found",
        relative_path.display()
    ))
}

fn read_merges_pending(workspace_root: &Path) -> Result<Vec<MergePending>, String> {
    let merges_pending_dir = workspace_root.join(".lsc/merge_pending");
    let mut res = Vec::new();
    match merges_pending_dir.read_dir() {
        Ok(dir_iterator) => {
            for entry_res in dir_iterator {
                match entry_res {
                    Ok(entry) => {
                        let parsed: serde_json::Result<MergePending> =
                            serde_json::from_str(&read_text_file(&entry.path())?);
                        match parsed {
                            Ok(merge) => {
                                res.push(merge);
                            }
                            Err(e) => {
                                return Err(format!("Error parsing {:?}: {}", entry.path(), e))
                            }
                        }
                    }
                    Err(e) => return Err(format!("Error reading merge pending entry: {}", e)),
                }
            }
        }
        Err(e) => {
            return Err(format!(
                "Error reading directory {:?}: {}",
                merges_pending_dir, e
            ))
        }
    }
    Ok(res)
}

pub fn find_merges_pending_command() -> Result<Vec<MergePending>, String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    read_merges_pending(&workspace_root)
}

fn find_file_hash_at_commit(
    repo: &Path,
    relative_path: &Path,
    commit_id: &str,
) -> Result<String, String> {
    let commit = read_commit(repo, commit_id)?;
    let root_tree = read_tree(repo, &commit.root_hash)?;
    let parent_dir = relative_path.parent().expect("no parent to path provided");
    let dir_tree = fetch_tree_subdir(repo, &root_tree, &parent_dir)?;
    let file_node = dir_tree.find_file_node(
        relative_path
            .file_name()
            .expect("no file name in path specified")
            .to_str()
            .expect("invalid file name"),
    )?;
    Ok(file_node.hash.clone())
}

fn run_merge_program(
    abs_path: &str,
    theirs_path: &str,
    base_path: &str,
    output_path: &str,
) -> Result<(), String> {
    let program = Path::new(r#"C:\Program Files\Beyond Compare 4\bcomp.exe"#);
    let left_title_arg = format!("/lefttitle={} [yours]", abs_path);
    match Command::new(program)
        .args(&[
            "/automerge",
            "/reviewconflicts",
            &left_title_arg,
            "/righttitle=[theirs]",
            "/centertitle=[base]",
            "/outputtitle=[output]",
            abs_path,    //left
            theirs_path, //right
            base_path,   //center
            output_path,
        ])
        .output()
    {
        Ok(output) => {
            println!("{}", std::str::from_utf8(&output.stdout).unwrap());
            println!("{}", std::str::from_utf8(&output.stderr).unwrap());
            if !output.status.success() {
                return Err(format!(
                    "merge program returned error code {}",
                    output.status.code().expect("error reading status code")
                ));
            }
        }
        Err(e) => {
            return Err(format!(
                "Error launching merge program {}: {}",
                program.display(),
                e
            ));
        }
    }
    Ok(())
}

pub fn merge_file_command(p: &Path) -> Result<(), String> {
    let abs_path = make_path_absolute(p);
    let workspace_root = find_workspace_root(&abs_path)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let repo = &workspace_spec.repository;
    let relative_path = path_relative_to(&abs_path, workspace_root)?;
    let merge_pending = find_merge_pending(&workspace_root, &relative_path)?;
    let tmp_dir = workspace_root.join(".lsc/tmp");
    let base_file_hash = find_file_hash_at_commit(
        &workspace_spec.repository,
        &relative_path,
        &merge_pending.base_commit_id,
    )?;
    let base_file_path = tmp_dir.join(&base_file_hash);
    download_blob(repo, &base_file_path, &base_file_hash)?;
    let theirs_file_hash = find_file_hash_at_commit(
        &workspace_spec.repository,
        &relative_path,
        &merge_pending.theirs_commit_id,
    )?;
    let theirs_file_path = tmp_dir.join(&theirs_file_hash);
    download_blob(repo, &theirs_file_path, &theirs_file_hash)?;
    let output_path = tmp_dir.join(format!("merge_output_{}", uuid::Uuid::new_v4().to_string()));
    run_merge_program(
        abs_path.to_str().unwrap(),
        theirs_file_path.to_str().unwrap(),
        base_file_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
    )?;
    if let Err(e) = fs::copy(&output_path, &abs_path){
        return Err(format!("Error copying {} to {}: {}", output_path.display(), abs_path.display(), e));
    }
    //todo: delete pending merge & temp files
    Ok(())
}
