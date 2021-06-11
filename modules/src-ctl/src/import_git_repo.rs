use crate::*;
use std::path::Path;

fn can_import_commit(repo: &Path, commit: &git2::Commit<'_>) -> bool {
    for parent_id in commit.parent_ids() {
        let id_str = parent_id.to_string();
        if !commit_exists(repo, &id_str) {
            return false;
        }
    }
    true
}

fn import_commit(
    _repo: &Path,
    workspace_root: &Path,
    git_repo: &git2::Repository,
    commit: &git2::Commit<'_>,
) -> Result<(), String> {
    println!("importing {:?}", commit);
    match commit.tree() {
        Ok(tree) => {
            if let Err(e) = tree.walk(git2::TreeWalkMode::PreOrder, |directory, entry| {
                let relative_path = Path::new(directory).join(entry.name().unwrap());
                let local_path = workspace_root.join(&relative_path);
                match entry.kind().unwrap() {
                    git2::ObjectType::Tree => {
                        if !local_path.exists() {
                            if let Err(e) = std::fs::create_dir(&local_path) {
                                println!(
                                    "Error creating local directory {}: {}",
                                    local_path.display(),
                                    e
                                );
                                return git2::TreeWalkResult::Abort;
                            }
                        }
                    }
                    git2::ObjectType::Blob => {
                        let blob = entry.to_object(git_repo).unwrap().into_blob().unwrap();
                        if let Err(e) = write_file(&local_path, blob.content()) {
                            println!(
                                "Error writing blob {} to {}: {}",
                                blob.id(),
                                local_path.display(),
                                e
                            );
                            return git2::TreeWalkResult::Abort;
                        }
                        // println!("{} {:?}", .display(), blob);
                    }
                    _ => {}
                }
                git2::TreeWalkResult::Ok
            }) {
                return Err(format!("walk failed for commit {:?}: {}", commit, e));
            }
        }
        Err(e) => {
            return Err(format!("Error getting tree for commit {:?}: {}", commit, e));
        }
    }
    Ok(())
}

fn import_commit_tree(
    repo: &Path,
    workspace_root: &Path,
    git_repo: &git2::Repository,
    root_commit: &git2::Commit<'_>,
) -> Result<(), String> {
    let mut stack = vec![root_commit.clone()];

    while !stack.is_empty() {
        let commit = stack.pop().unwrap();
        let commit_id = root_commit.id().to_string();
        if !commit_exists(repo, &commit_id) {
            if can_import_commit(repo, &commit) {
                import_commit(repo, workspace_root, git_repo, &commit)?;
                return Ok(()); //hack
            } else {
                //put it back in the stack with its parents
                stack.push(commit.clone());
                for parent in commit.parents() {
                    stack.push(parent);
                }
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
            import_commit_tree(repo, workspace_root, git_repo, &commit)?;
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
