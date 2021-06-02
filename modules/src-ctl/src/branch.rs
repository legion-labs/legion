use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

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
    let mut files_present: BTreeMap<String, String> = BTreeMap::new();
    let mut dirs_present: BTreeMap<String, String> = BTreeMap::new();
    if !current_tree_hash.is_empty() {
        let current_tree = read_tree(repo, current_tree_hash)?;
        for file_node in &current_tree.file_nodes {
            files_present.insert(file_node.name.clone(), file_node.hash.clone());
        }

        for dir_node in &current_tree.directory_nodes {
            dirs_present.insert(dir_node.name.clone(), dir_node.hash.clone());
        }
    }

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
                    println!("{}", e);
                }
            }
        }
    }

    //those files were not matched, delete them
    for k in files_present.keys() {
        let abs_path = workspace_root.join(relative_path_tree).join(&k);
        match sync_file(repo, &abs_path, "") {
            Ok(message) => {
                println!("{}", message);
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    for new_dir_node in &new_tree.directory_nodes {
        let present_hash = match dirs_present.get(&new_dir_node.name) {
            Some(hash) => {
                let res = hash.clone();
                dirs_present.remove(&new_dir_node.name);
                res
            }
            None => String::new(),
        };
        let relative_sub_dir = relative_path_tree.join(&new_dir_node.name);
        let abs_dir = workspace_root.join(&relative_sub_dir);
        if !abs_dir.exists() {
            if let Err(e) = fs::create_dir(&abs_dir) {
                println!("Error creating directory {}: {}", abs_dir.display(), e);
            }
        }
        if new_dir_node.hash != present_hash {
            if let Err(e) = sync_tree_diff(
                &repo,
                &present_hash,
                &new_dir_node.hash,
                &relative_sub_dir,
                &workspace_root,
            ) {
                println!("{}", e);
            }
        }
    }
    //delete the contents of the directories that were not matched
    for (name, hash) in dirs_present {
        let path = workspace_root.join(&relative_path_tree).join(name);
        match remove_dir_rec(&repo, &path, &hash) {
            Ok(messages) => {
                if !messages.is_empty() {
                    println!("{}", messages);
                }
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

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

pub fn list_branches_command() -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let repo = &workspace_spec.repository;
    let branches_dir = repo.join("branches");
    match branches_dir.read_dir() {
        Ok(dir_iterator) => {
            for entry_res in dir_iterator {
                match entry_res {
                    Ok(entry) => {
                        let parsed: serde_json::Result<Branch> =
                            serde_json::from_str(&read_text_file(&entry.path())?);
                        match parsed {
                            Ok(branch) => {
                                println!(
                                    "{} head:{} parent:{}",
                                    branch.name, branch.head, branch.parent
                                );
                            }
                            Err(e) => {
                                return Err(format!(
                                    "Error parsing {}: {}",
                                    entry.path().display(),
                                    e
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        return Err(format!("Error reading branch entry: {}", e));
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!(
                "Error reading {} directory: {}",
                branches_dir.display(),
                e
            ));
        }
    }
    Ok(())
}

fn find_latest_common_ancestor(
    sequence_branch_one: &[Commit],
    set_branch_two: &BTreeSet<String>,
) -> Option<String> {
    // if the times are reliable we can cut short this search
    for c in sequence_branch_one {
        if set_branch_two.contains(&c.id) {
            return Some(c.id.clone());
        }
    }
    None
}

fn change_file_to(
    repo: &Path,
    relative_path: &Path,
    workspace_root: &Path,
    hash_to_sync: &str,
) -> Result<String, String> {
    let local_path = workspace_root.join(relative_path);
    if local_path.exists() {
        let local_hash = compute_file_hash(&local_path)?;
        if local_hash == hash_to_sync {
            return Ok(format!("Verified {}", local_path.display()));
        }
        if hash_to_sync.is_empty() {
            delete_file_command(&local_path)?;
            return Ok(format!("Deleted {}", local_path.display()));
        }
        edit_file_command(&local_path)?;
        if let Err(e) = download_blob(&repo, &local_path, &hash_to_sync) {
            return Err(format!(
                "Error downloading {} {}: {}",
                local_path.display(),
                &hash_to_sync,
                e
            ));
        }
        if let Err(e) = make_file_read_only(&local_path, true) {
            return Err(e);
        }
        return Ok(format!("Updated {}", local_path.display()));
    } else {
        //no local file
        if hash_to_sync.is_empty() {
            return Ok(format!("Verified {}", local_path.display()));
        }
        if let Err(e) = download_blob(&repo, &local_path, &hash_to_sync) {
            return Err(format!(
                "Error downloading {} {}: {}",
                local_path.display(),
                &hash_to_sync,
                e
            ));
        }
        if let Err(e) = make_file_read_only(&local_path, true) {
            return Err(e);
        }
        track_new_file(&local_path)?;
        return Ok(format!("Added {}", local_path.display()));
    }
}

pub fn merge_branch_command(name: &str) -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let repo = &workspace_spec.repository;
    let branch_to_merge = read_branch_from_repo(&repo, &name)?;
    let current_branch = read_current_branch(&workspace_root)?;
    let mut latest_branch = read_branch_from_repo(&repo, &current_branch.name)?;

    let branch_commits = find_branch_commits(&repo, &branch_to_merge)?;
    let mut branch_commit_ids_set: BTreeSet<String> = BTreeSet::new();
    for c in &branch_commits {
        branch_commit_ids_set.insert(c.id.clone());
    }
    if branch_commit_ids_set.contains(&latest_branch.head) {
        //fast forward case
        latest_branch.head = branch_to_merge.head;
        save_current_branch(&workspace_root, &latest_branch)?;
        save_branch_to_repo(&repo, &latest_branch)?;
        println!("Fast-forward merge: branch updated, synching");
        return sync_command();
    }

    if current_branch.head != latest_branch.head {
        return Err(String::from(
            "Workspace not up to date, sync to latest before merge",
        ));
    }

    let mut errors: Vec<String> = Vec::new();
    let latest_commits = find_branch_commits(&repo, &latest_branch)?;
    if let Some(common_ancestor_id) =
        find_latest_common_ancestor(&latest_commits, &branch_commit_ids_set)
    {
        let mut modified_in_current: BTreeMap<PathBuf, String> = BTreeMap::new();
        for commit in &latest_commits {
            if commit.id == common_ancestor_id {
                break;
            }
            for change in &commit.changes {
                modified_in_current
                    .entry(change.relative_path.clone())
                    .or_insert_with(|| change.hash.clone());
            }
        }

        let mut to_update: BTreeMap<PathBuf, String> = BTreeMap::new();
        for commit in &branch_commits {
            if commit.id == common_ancestor_id {
                break;
            }
            for change in &commit.changes {
                to_update
                    .entry(change.relative_path.clone())
                    .or_insert_with(|| change.hash.clone());
            }
        }

        for (path, hash) in to_update.iter() {
            if modified_in_current.contains_key(path) {
                //todo: support conflicts
                return Err(format!(
                    "merge aborted, conflict found with {}",
                    path.display()
                ));
            }
            match change_file_to(&repo, &path, &workspace_root, &hash) {
                Ok(message) => {
                    println!("{}", message);
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }
    } else {
        return Err(String::from(
            "Error finding common ancestor for branch merge",
        ));
    }

    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    println!("merge completed, ready to commit");
    Ok(())
}
