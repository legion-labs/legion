use crate::*;
use std::path::Path;

pub fn revert_glob_command(pattern: &str) -> Result<(), String> {
    let mut nb_errors = 0;
    match glob::Pattern::new(pattern) {
        Ok(matcher) => {
            let current_dir = std::env::current_dir().unwrap();
            let workspace_root = find_workspace_root(&current_dir)?;
            for change in read_local_changes(&workspace_root)? {
                if matcher.matches(&change.relative_path) {
                    println!("reverting {}", change.relative_path);
                    let local_file_path = workspace_root.join(change.relative_path);
                    if let Err(e) = revert_file_command(&local_file_path) {
                        println!("{}", e);
                        nb_errors += 1;
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!("Error parsing glob pattern: {}", e));
        }
    }
    if nb_errors == 0 {
        Ok(())
    } else {
        Err(format!("{} errors", nb_errors))
    }
}

pub fn revert_file_command(path: &Path) -> Result<(), String> {
    let abs_path = make_path_absolute(path);
    let workspace_root = find_workspace_root(&abs_path)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let relative_path = make_canonical_relative_path(&workspace_root, &abs_path)?;
    let local_change = match find_local_change(&workspace_root, &relative_path) {
        SearchResult::Ok(change) => change,
        SearchResult::Err(e) => {
            return Err(format!("Error searching in local changes: {}", e));
        }
        SearchResult::None => {
            return Err(format!("Error local change {} not found", relative_path));
        }
    };
    let parent_dir = Path::new(&relative_path)
        .parent()
        .expect("no parent to path provided");
    let workspace_branch = read_current_branch(&workspace_root)?;
    let current_commit = read_commit(&workspace_spec.repository, &workspace_branch.head)?;
    let root_tree = read_tree(&workspace_spec.repository, &current_commit.root_hash)?;
    let dir_tree = fetch_tree_subdir(&workspace_spec.repository, &root_tree, parent_dir)?;

    if local_change.change_type != "add" {
        let file_node = dir_tree.find_file_node(
            abs_path
                .file_name()
                .expect("no file name in path specified")
                .to_str()
                .expect("invalid file name"),
        )?;
        download_blob(&workspace_spec.repository, &abs_path, &file_node.hash)?;
        make_file_read_only(&abs_path, true)?;
    }
    clear_local_change(&workspace_root, &local_change)?;
    match find_resolve_pending(&workspace_root, &relative_path) {
        SearchResult::Ok(resolve_pending) => {
            clear_resolve_pending(&workspace_root, &resolve_pending)
        }
        SearchResult::Err(e) => Err(format!(
            "Error finding resolve pending for file {}: {}",
            relative_path, e
        )),
        SearchResult::None => Ok(()),
    }
}
