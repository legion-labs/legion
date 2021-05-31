use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Branch {
    pub name: String,
    pub head: String, //commit id
    pub parent: String,
}

impl Branch {
    pub fn new(name: String, head: String, parent: String) -> Branch {
        Branch { name, head, parent }
    }
}

fn write_branch_spec(file_path: &Path, branch: &Branch) -> Result<(), String> {
    match serde_json::to_string(branch) {
        Ok(json) => write_file(&file_path, json.as_bytes()),
        Err(e) => Err(format!("Error formatting branch {:?}: {}", branch, e)),
    }
}

pub fn save_new_branch_to_repo(repo: &Path, branch: &Branch) -> Result<(), String> {
    let file_path = repo.join("branches").join(branch.name.to_owned() + ".json");
    match serde_json::to_string(branch) {
        Ok(json) => write_new_file(&file_path, json.as_bytes()),
        Err(e) => Err(format!("Error formatting branch {:?}: {}", branch, e)),
    }
}

pub fn save_branch_to_repo(repo: &Path, branch: &Branch) -> Result<(), String> {
    let file_path = repo.join("branches").join(branch.name.to_owned() + ".json");
    write_branch_spec(&file_path, branch)
}

pub fn save_current_branch(workspace_root: &Path, branch: &Branch) -> Result<(), String> {
    let file_path = workspace_root.join(".lsc/branch.json");
    write_branch_spec(&file_path, branch)
}

pub fn read_current_branch(workspace_root: &Path) -> Result<Branch, String> {
    let file_path = workspace_root.join(".lsc/branch.json");
    read_branch(&file_path)
}

pub fn read_branch_from_repo(repo: &Path, name: &str) -> Result<Branch, String> {
    let file_path = repo.join("branches").join(name.to_owned() + ".json");
    read_branch(&file_path)
}

pub fn read_branch(branch_file_path: &Path) -> Result<Branch, String> {
    let parsed: serde_json::Result<Branch> =
        serde_json::from_str(&read_text_file(branch_file_path)?);
    match parsed {
        Ok(branch) => Ok(branch),
        Err(e) => Err(format!(
            "Error reading branch spec {}: {}",
            branch_file_path.display(),
            e
        )),
    }
}

pub fn create_branch_command(name: &str) -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let old_branch = read_current_branch(&workspace_root)?;
    let new_branch = Branch::new(String::from(name), old_branch.head.clone(), old_branch.name);
    save_new_branch_to_repo(&workspace_spec.repository, &new_branch)?;
    save_current_branch(&workspace_root, &new_branch)
}

fn sync_tree_diff(
    repo: &Path,
    current_tree_hash: &str,
    new_tree_hash: &str,
    relative_path_tree: &Path,
    workspace_root: &Path,
) -> Result<(), String> {
    let current_tree = read_tree(repo, current_tree_hash)?;
    let mut files_present: BTreeMap<String, String> = BTreeMap::new();
    for file_node in &current_tree.file_nodes {
        files_present.insert(file_node.name.clone(), file_node.hash.clone());
    }

    let mut errors: Vec<String> = Vec::new();
    let new_tree = read_tree(repo, new_tree_hash)?;
    for new_file_node in &new_tree.file_nodes {
        let present_hash = match files_present.get(&new_file_node.name) {
            Some(hash) => {
                let res = hash.clone();
                files_present.remove(&new_file_node.name);
                res
            }
            None => String::new(),
        };
        if new_file_node.hash != present_hash {
            match sync_file(
                repo,
                &workspace_root
                    .join(relative_path_tree)
                    .join(&new_file_node.name),
                &new_file_node.hash,
            ) {
                Ok(message) => {
                    println!("{}", message);
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }
    }

    //those files were not matched, delete them
    for k in files_present.keys() {
        match sync_file(repo, &workspace_root.join(relative_path_tree).join(&k), "") {
            Ok(message) => {
                println!("{}", message);
            }
            Err(e) => {
                errors.push(e);
            }
        }
    }

    //todo: deal with subdirectories

    Ok(())
}

pub fn switch_branch_command(name: &str) -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let repo = &workspace_spec.repository;
    let old_branch = read_current_branch(&workspace_root)?;
    let old_commit = read_commit(&repo, &old_branch.head)?;
    let new_branch = read_branch_from_repo(&repo, name)?;
    let new_commit = read_commit(&repo, &new_branch.head)?;
    save_current_branch(&workspace_root, &new_branch)?;
    sync_tree_diff(
        &repo,
        &old_commit.root_hash,
        &new_commit.root_hash,
        Path::new(""),
        &workspace_root,
    )
}
