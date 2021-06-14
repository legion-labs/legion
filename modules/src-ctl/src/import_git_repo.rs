use crate::*;
use std::path::Path;

fn copy_git_blob(
    git_repo: &git2::Repository,
    blob_oid: git2::Oid,
    destination: &Path,
) -> Result<(), String> {
    let parent_dir = destination.parent().unwrap();
    if !parent_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(parent_dir) {
            return Err(format!(
                "Error creating directory {}: {}",
                parent_dir.display(),
                e
            ));
        }
    }

    match git_repo.find_blob(blob_oid) {
        Ok(blob) => {
            if let Err(e) = write_file(destination, blob.content()) {
                return Err(format!(
                    "Error writing blob {} to {}: {}",
                    blob.id(),
                    destination.display(),
                    e
                ));
            }
        }
        Err(e) => {
            return Err(format!("Error in find_blob for {}: {}", blob_oid, e));
        }
    }
    Ok(())
}

fn add_file_from_git(
    workspace_root: &Path,
    git_repo: &git2::Repository,
    new_file: &git2::DiffFile<'_>,
) -> Result<(), String> {
    let relative_path = new_file.path().unwrap();
    let local_path = workspace_root.join(relative_path);
    if let Err(e) = copy_git_blob(git_repo, new_file.id(), &local_path) {
        return Err(format!(
            "Error copy git blob {} to {}: {}",
            new_file.id(),
            local_path.display(),
            e
        ));
    }
    track_new_file(&local_path)
}

fn edit_file_from_git(
    workspace_root: &Path,
    git_repo: &git2::Repository,
    new_file: &git2::DiffFile<'_>,
) -> Result<(), String> {
    let relative_path = new_file.path().unwrap();
    let local_path = workspace_root.join(relative_path);

    if let Err(e) = edit_file_command(&local_path) {
        return Err(format!("Error editing {}: {}", local_path.display(), e));
    }

    if let Err(e) = copy_git_blob(git_repo, new_file.id(), &local_path) {
        return Err(format!(
            "Error copy git blob {} to {}: {}",
            new_file.id(),
            local_path.display(),
            e
        ));
    }
    Ok(())
}

fn import_commit_diff(
    workspace_root: &Path,
    diff: &git2::Diff<'_>,
    git_repo: &git2::Repository,
) -> Result<(), String> {
    let mut errors: Vec<String> = Vec::new();
    if let Err(e) = diff.foreach(
        &mut |delta, _progress| {
            match delta.status() {
                git2::Delta::Added => {
                    let new_file = delta.new_file();
                    if let Err(e) = add_file_from_git(workspace_root, git_repo, &new_file) {
                        errors.push(format!("Error adding file {:?}: {}", new_file, e));
                    } else {
                        println!("added {}", new_file.path().unwrap().display());
                    }
                }
                git2::Delta::Deleted => {
                    let old_file = delta.old_file();
                    let local_file = workspace_root.join(old_file.path().unwrap());
                    if let Err(e) = delete_file_command(&local_file) {
                        errors.push(format!(
                            "Error deleting file {:?}: {}",
                            local_file.display(),
                            e
                        ));
                    } else {
                        println!("deleted {}", old_file.path().unwrap().display());
                    }
                }
                git2::Delta::Modified => {
                    let new_file = delta.new_file();
                    if let Err(e) = edit_file_from_git(workspace_root, git_repo, &new_file) {
                        errors.push(format!("Error modifying file {:?}: {}", new_file, e));
                    } else {
                        println!("modified {}", new_file.path().unwrap().display());
                    }
                }
                //todo: make a test case for those
                // git2::Delta::Renamed => {}
                // git2::Delta::Copied => {}
                status => {
                    errors.push(format!(
                        "Skipping change of type {:?}. Old file: {}. New file: {}.",
                        status,
                        delta.old_file().path().unwrap().display(),
                        delta.new_file().path().unwrap().display()
                    ));
                }
            }
            true //continue foreach
        },
        None,
        None,
        None,
    ) {
        if !errors.is_empty() {
            return Err(errors.join("\n"));
        }
        return Err(format!("Error iterating in diff: {}", e));
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(())
}

// import_commit_sequence walks this history by traversing the first parent only
// and stops when a commit has been previously imported or when the root is found (has no parent).
// We could try to import the whole commit tree but for our purposes it's not necessary
// and is significantly more complex.
// One alternative would be to find the shortest path between the last integrated commit and the
// top of the branch.
fn import_commit_sequence(
    repo: &Path,
    workspace_root: &Path,
    git_repo: &git2::Repository,
    root_commit: &git2::Commit<'_>,
) -> Result<(), String> {
    let mut stack = vec![root_commit.clone()];
    let mut reference_index = git2::Index::new().unwrap();
    loop {
        let commit = stack.last().unwrap();
        let commit_id = root_commit.id().to_string();
        if commit_exists(repo, &commit_id) {
            match commit.tree() {
                Ok(tree) => {
                    if let Err(e) = reference_index.read_tree(&tree) {
                        return Err(format!(
                            "Error reading tree from commit {:?} into index: {}",
                            commit, e
                        ));
                    }
                }
                Err(e) => {
                    return Err(format!(
                        "Error getting tree from commit {:?}: {}",
                        commit, e
                    ));
                }
            }
            break;
        }
        if commit.parent_count() == 0 {
            break;
        }
        match commit.parent(0) {
            Ok(parent) => {
                stack.push(parent);
            }
            Err(e) => {
                return Err(format!(
                    "Error fetching commit parent for {:?}: {}",
                    commit, e
                ));
            }
        }
    }

    while !stack.is_empty() {
        let commit = stack.pop().unwrap();
        let message = String::from_utf8_lossy(commit.message_bytes());

        println!("importing commit {}: {}", commit.id(), message);
        match commit.tree() {
            Ok(tree) => {
                let mut current_index = git2::Index::new().unwrap();
                if let Err(e) = current_index.read_tree(&tree) {
                    return Err(format!(
                        "Error reading tree from commit {:?} into index: {}",
                        commit, e
                    ));
                }
                match git_repo.diff_index_to_index(&reference_index, &current_index, None) {
                    Ok(diff) => {
                        if let Err(e) = import_commit_diff(workspace_root, &diff, git_repo) {
                            return Err(format!("Error importing {:?}: {}", commit, e));
                        }
                        if let Err(e) = commit_local_changes(workspace_root, &message) {
                            return Err(format!("Error recording {:?}: {}", commit, e));
                        }
                    }
                    Err(e) => {
                        return Err(format!("Error in diff for {:?}: {}", commit, e));
                    }
                }

                reference_index = current_index;
            }
            Err(e) => {
                return Err(format!(
                    "Error getting tree from commit {:?}: {}",
                    commit, e
                ));
            }
        }
    }
    Ok(())
}

fn import_branch(
    repo: &Path,
    workspace_root: &Path,
    git_repo: &git2::Repository,
    branch: &git2::Branch<'_>,
) -> Result<(), String> {
    let branch_name = branch.name().unwrap().unwrap();
    println!("importing branch {}", branch_name);

    match find_branch(repo, branch_name) {
        SearchResult::Ok(_branch) => {
            println!("branch already exists");
        }
        SearchResult::Err(e) => {
            return Err(format!("Error reading local branch {}: {}", branch_name, e));
        }
        SearchResult::None => {
            panic!("branch creation not supported");
        }
    }

    match branch.get().peel_to_commit() {
        Ok(commit) => {
            import_commit_sequence(repo, workspace_root, git_repo, &commit)?;
        }
        Err(e) => {
            return Err(format!("Branch reference is not a commit: {}", e));
        }
    }
    Ok(())
}

pub fn import_git_repo_command(git_root_path: &Path) -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let repo = &workspace_spec.repository;
    match git2::Repository::open(git_root_path) {
        Ok(git_repo) => {
            println!("git repository state: {:?}", git_repo.state());
            //todo: instead of discovering branches, the user should specify which one to import
            match git_repo.branches(Some(git2::BranchType::Local)) {
                Ok(branches) => {
                    for branch_result in branches {
                        match branch_result {
                            Ok((branch, _branch_type)) => {
                                import_branch(repo, &workspace_root, &git_repo, &branch)?;
                            }
                            Err(e) => {
                                return Err(format!("Error iterating in branches: {}", e));
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(format!("Error listing branches: {}", e));
                }
            }
        }
        Err(e) => {
            return Err(format!(
                "Error opening git repository at {}: {}",
                git_root_path.display(),
                e
            ));
        }
    }
    Ok(())
}
