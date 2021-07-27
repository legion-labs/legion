use crate::*;
use std::path::Path;

fn format_commit(c: &git2::Commit<'_>) -> String {
    format!("{} {}", c.id(), c.summary().unwrap())
}

fn copy_git_blob(
    git_repo: &git2::Repository,
    blob_oid: git2::Oid,
    destination: &Path,
) -> Result<(), String> {
    match git_repo.find_blob(blob_oid) {
        Ok(blob) => {
            let content = blob.content();
            if let Err(e) = write_file(destination, content) {
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

async fn add_file_from_git(
    workspace_connection: &mut LocalWorkspaceConnection,
    repo_connection: &mut RepositoryConnection,
    git_repo: &git2::Repository,
    new_file: &git2::DiffFile<'_>,
) -> Result<(), String> {
    let relative_path = new_file.path().unwrap();
    let workspace_root = workspace_connection.workspace_path().to_path_buf();
    let local_path = workspace_root.join(relative_path);
    let canonical_relative_path = make_canonical_relative_path(&workspace_root, &local_path)?;

    match find_local_change(workspace_connection, &canonical_relative_path) {
        Ok(Some(change)) => {
            if change.change_type == ChangeType::Delete {
                println!("adding of file being deleted - reverting change and editing");
                revert_file_command(&local_path)?;
                return edit_file_from_git(
                    workspace_connection,
                    repo_connection,
                    git_repo,
                    new_file,
                )
                .await;
            }
            return Err(format!(
                "Error: {} already tracked for {:?}",
                change.relative_path, change.change_type
            ));
        }
        Err(e) => {
            return Err(format!("Error searching in local changes: {}", e));
        }
        Ok(None) => { //all is good
        }
    }

    if local_path.exists() {
        return Err(String::from("local file already exists"));
    }
    if let Err(e) = copy_git_blob(git_repo, new_file.id(), &local_path) {
        return Err(format!(
            "Error copy git blob {} to {}: {}",
            new_file.id(),
            local_path.display(),
            e
        ));
    }
    track_new_file_command(&local_path)
}

async fn edit_file_from_git(
    workspace_connection: &mut LocalWorkspaceConnection,
    repo_connection: &mut RepositoryConnection,
    git_repo: &git2::Repository,
    new_file: &git2::DiffFile<'_>,
) -> Result<(), String> {
    let relative_path = new_file.path().unwrap();
    let local_path = workspace_connection.workspace_path().join(relative_path);

    if let Err(e) = edit_file(workspace_connection, repo_connection, &local_path).await {
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
    workspace_connection: &mut LocalWorkspaceConnection,
    repo_connection: &mut RepositoryConnection,
    runtime: &tokio::runtime::Runtime,
    diff: &git2::Diff<'_>,
    git_repo: &git2::Repository,
) -> Result<(), String> {
    let mut errors: Vec<String> = Vec::new();
    let workspace_root = workspace_connection.workspace_path().to_path_buf();
    //process deletes first
    //because git is case-sensitive a file can be seen as being removed and added in the same commit
    if let Err(e) = diff.foreach(
        &mut |delta, _progress| {
            if let git2::Delta::Deleted = delta.status() {
                let old_file = delta.old_file();
                let local_file = workspace_root.join(old_file.path().unwrap());
                println!("deleting {}", old_file.path().unwrap().display());
                if let Err(e) = delete_file_command(&local_file) {
                    let message = format!("Error deleting file {}: {}", local_file.display(), e);
                    println!("{}", message);
                    errors.push(message);
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

    if let Err(e) = diff.foreach(
        &mut |delta, _progress| {
            match delta.status() {
                git2::Delta::Added => {
                    let new_file = delta.new_file();
                    let path = new_file.path().unwrap();
                    println!("adding {}", path.display());
                    if let Err(e) = runtime.block_on(add_file_from_git(
                        workspace_connection,
                        repo_connection,
                        git_repo,
                        &new_file,
                    )) {
                        let message = format!("Error adding file {}: {}", path.display(), e);
                        println!("{}", message);
                        errors.push(message);
                    }
                }
                git2::Delta::Deleted => {}
                git2::Delta::Modified => {
                    let new_file = delta.new_file();
                    let path = new_file.path().unwrap();
                    println!("modifying {}", path.display());
                    if let Err(e) = runtime.block_on(edit_file_from_git(
                        workspace_connection,
                        repo_connection,
                        git_repo,
                        &new_file,
                    )) {
                        let message = format!("Error modifying file {}: {}", path.display(), e);
                        println!("{}", message);
                        errors.push(message);
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
    repo_connection: &mut RepositoryConnection,
    workspace_connection: &mut LocalWorkspaceConnection,
    runtime: &tokio::runtime::Runtime,
    git_repo: &git2::Repository,
    root_commit: &git2::Commit<'_>,
) -> Result<(), String> {
    let mut stack = vec![root_commit.clone()];
    let mut reference_index = git2::Index::new().unwrap();
    loop {
        let commit = stack.pop().unwrap();
        let commit_id = commit.id().to_string();
        if commit_exists(repo_connection, &commit_id) {
            match commit.tree() {
                Ok(tree) => {
                    if let Err(e) = reference_index.read_tree(&tree) {
                        return Err(format!(
                            "Error reading tree from commit {} into index: {}",
                            format_commit(&commit),
                            e
                        ));
                    }
                }
                Err(e) => {
                    return Err(format!(
                        "Error getting tree from commit {}: {}",
                        format_commit(&commit),
                        e
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
                stack.push(commit);
                stack.push(parent);
            }
            Err(e) => {
                return Err(format!(
                    "Error fetching commit parent for {}: {}",
                    format_commit(&commit),
                    e
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
                        "Error reading tree from commit {} into index: {}",
                        format_commit(&commit),
                        e
                    ));
                }
                match git_repo.diff_index_to_index(&reference_index, &current_index, None) {
                    Ok(diff) => {
                        if let Err(e) = import_commit_diff(
                            workspace_connection,
                            repo_connection,
                            runtime,
                            &diff,
                            git_repo,
                        ) {
                            return Err(format!(
                                "Error importing {}: {}",
                                format_commit(&commit),
                                e
                            ));
                        }
                        let commit_id = commit.id().to_string();
                        println!("recording commit {}: {}", commit_id, message);
                        if let Err(e) =
                            commit_local_changes(workspace_connection, &commit_id, &message)
                        {
                            return Err(format!(
                                "Error recording {}: {}",
                                format_commit(&commit),
                                e
                            ));
                        }
                    }
                    Err(e) => {
                        return Err(format!(
                            "Error in diff for {}: {}",
                            format_commit(&commit),
                            e
                        ));
                    }
                }

                reference_index = current_index;
            }
            Err(e) => {
                return Err(format!(
                    "Error getting tree from {}: {}",
                    format_commit(&commit),
                    e
                ));
            }
        }
    }
    Ok(())
}

fn import_branch(
    repo_connection: &mut RepositoryConnection,
    workspace_connection: &mut LocalWorkspaceConnection,
    runtime: &tokio::runtime::Runtime,
    git_repo: &git2::Repository,
    branch: &git2::Branch<'_>,
) -> Result<(), String> {
    let branch_name = branch.name().unwrap().unwrap();
    println!("importing branch {}", branch_name);

    match find_branch(repo_connection, branch_name) {
        Ok(Some(_branch)) => {
            println!("branch already exists");
        }
        Err(e) => {
            return Err(format!("Error reading local branch {}: {}", branch_name, e));
        }
        Ok(None) => {
            panic!("branch creation not supported");
        }
    }

    match branch.get().peel_to_commit() {
        Ok(commit) => {
            import_commit_sequence(
                repo_connection,
                workspace_connection,
                runtime,
                git_repo,
                &commit,
            )?;
        }
        Err(e) => {
            return Err(format!("Branch reference is not a commit: {}", e));
        }
    }
    Ok(())
}

pub fn import_git_repo_command(git_root_path: &Path, branch: Option<&str>) -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let mut repo_connection = tokio_runtime.block_on(connect_to_server(&workspace_spec))?;
    match git2::Repository::open(git_root_path) {
        Ok(git_repo) => {
            println!("git repository state: {:?}", git_repo.state());
            // todo: instead of discovering branches, the user should specify which one to import
            // https://github.com/legion-labs/legion/issues/21
            match git_repo.branches(Some(git2::BranchType::Local)) {
                Ok(mut branches) => {
                    let git_branch = match branch {
                        Some(branch) => branches.find(|result| {
                            if let Ok((git_branch, _branch_type)) = result {
                                if let Some(branch_shorthand) = git_branch.get().shorthand() {
                                    return branch_shorthand.eq(branch);
                                }
                            }
                            false
                        }),
                        None => {
                            // if no branch specified, then just import the first valid discovered branch
                            branches.next()
                        }
                    };

                    match git_branch {
                        Some(branch_result) => match branch_result {
                            Ok((git_branch, _branch_type)) => {
                                import_branch(
                                    &mut repo_connection,
                                    &mut workspace_connection,
                                    &tokio_runtime,
                                    &git_repo,
                                    &git_branch,
                                )?;
                            }
                            Err(e) => {
                                return Err(format!("Error iterating in branches: {}", e));
                            }
                        },
                        None => {
                            match branch {
                                Some(branch) => {
                                    return Err(format!(
                                        "Cannot find branch '{}' in repository",
                                        branch
                                    ));
                                }
                                None => {
                                    // ? not sure if should return Ok ?
                                    return Err("No valid branches found in repository".to_string());
                                }
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
