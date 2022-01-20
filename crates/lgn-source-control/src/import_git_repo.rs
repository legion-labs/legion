use std::path::Path;

use anyhow::{Context, Result};
use lgn_tracing::span_fn;

use crate::{
    commit_local_changes, delete_local_file, edit_file, make_canonical_relative_path, revert_file,
    track_new_file, write_file, ChangeType, Workspace,
};

fn format_commit(c: &git2::Commit<'_>) -> String {
    format!("{} {}", c.id(), c.summary().unwrap())
}

fn copy_git_blob(
    git_repo: &git2::Repository,
    blob_oid: git2::Oid,
    destination: &Path,
) -> Result<()> {
    let blob = git_repo
        .find_blob(blob_oid)
        .context(format!("failed to find blob {}", blob_oid))?;
    let content = blob.content();

    write_file(destination, content).context(format!(
        "error writing blob {} to {}",
        blob_oid,
        destination.display()
    ))
}

async fn add_file_from_git(
    workspace: &Workspace,
    git_repo: &git2::Repository,
    new_file_path: impl AsRef<Path>,
    new_file_id: git2::Oid,
) -> Result<()> {
    let local_path = workspace.root.join(new_file_path.as_ref());
    let canonical_relative_path = make_canonical_relative_path(&workspace.root, &local_path)?;

    if let Some(change) = workspace
        .backend
        .find_local_change(&canonical_relative_path)
        .await
        .context("searching in local changes")?
    {
        if change.change_type != ChangeType::Delete {
            anyhow::bail!(
                "{} is already tracked for {:?}",
                change.relative_path,
                change.change_type
            );
        }

        println!("adding of file being deleted - reverting change and editing");
        revert_file(workspace, &local_path).await?;

        return edit_file_from_git(workspace, git_repo, new_file_path, new_file_id).await;
    }

    if local_path.exists() {
        anyhow::bail!("local file already exists: {}", local_path.display());
    }

    copy_git_blob(git_repo, new_file_id, &local_path).context(format!(
        "failed to copy git blob {} to {}",
        new_file_id,
        local_path.display()
    ))?;

    track_new_file(workspace, &local_path).await
}

async fn edit_file_from_git(
    workspace: &Workspace,
    git_repo: &git2::Repository,
    new_file_path: impl AsRef<Path>,
    new_file_id: git2::Oid,
) -> Result<()> {
    let local_path = workspace.root.join(new_file_path.as_ref());

    edit_file(workspace, &local_path)
        .await
        .context(format!("editing: {}", local_path.display()))?;

    copy_git_blob(git_repo, new_file_id, &local_path).context(format!(
        "failed to copy git blob {} to {}",
        new_file_id,
        local_path.display()
    ))
}

async fn import_commit_diff(
    workspace: &Workspace,
    diff: &git2::Diff<'_>,
    git_repo: &git2::Repository,
) -> Result<()> {
    // Let's process deletes first.
    //
    // Since git is case-sensitive, a file can be seen as being removed and added in
    // the same commit.
    let mut files_to_delete = vec![];

    diff.foreach(
        &mut |delta, _progress| {
            if let git2::Delta::Deleted = delta.status() {
                let old_file = delta.old_file();
                let local_file = workspace.root.join(old_file.path().unwrap());
                files_to_delete.push(local_file);
            }

            true //continue foreach
        },
        None,
        None,
        None,
    )
    .context("failed to iterate over diff")?;

    for local_file in files_to_delete {
        println!("deleting {}", local_file.display());

        delete_local_file(workspace, &local_file)
            .await
            .context(format!("error deleting file: {}", local_file.display()))?;
    }

    let mut files_to_add = vec![];
    let mut files_to_edit = vec![];

    diff.foreach(
        &mut |delta, _progress| {
            match delta.status() {
                git2::Delta::Added => {
                    let new_file = delta.new_file();
                    files_to_add.push((new_file.path().unwrap().to_path_buf(), new_file.id()));
                }
                git2::Delta::Deleted => {}
                git2::Delta::Modified => {
                    let new_file = delta.new_file();
                    files_to_edit.push((new_file.path().unwrap().to_path_buf(), new_file.id()));
                }
                //todo: make a test case for those
                // git2::Delta::Renamed => {}
                // git2::Delta::Copied => {}
                status => {
                    println!(
                        "Skipping change of type {:?}. Old file: {}. New file: {}.",
                        status,
                        delta.old_file().path().unwrap().display(),
                        delta.new_file().path().unwrap().display()
                    );

                    return false;
                }
            }
            true //continue foreach
        },
        None,
        None,
        None,
    )
    .context("failed to iterate over diff")?;

    for (new_file_path, new_file_id) in files_to_add {
        println!("adding {}", new_file_path.display());

        add_file_from_git(workspace, git_repo, &new_file_path, new_file_id)
            .await
            .context(format!("error adding file: {}", new_file_path.display()))?;
    }

    for (new_file_path, new_file_id) in files_to_edit {
        println!("modifying {}", new_file_path.display());

        edit_file_from_git(workspace, git_repo, &new_file_path, new_file_id)
            .await
            .context(format!("error modifying file: {}", new_file_path.display()))?;
    }

    Ok(())
}

// import_commit_sequence walks this history by traversing the first parent only
// and stops when a commit has been previously imported or when the root is
// found (has no parent). We could try to import the whole commit tree but for
// our purposes it's not necessary and is significantly more complex.
// One alternative would be to find the shortest path between the last
// integrated commit and the top of the branch.
async fn import_commit_sequence(
    workspace: &Workspace,
    git_repo: &git2::Repository,
    root_commit: &git2::Commit<'_>,
) -> Result<()> {
    let mut stack = vec![root_commit.clone()];
    let mut reference_index = git2::Index::new().unwrap();

    loop {
        let commit = stack.pop().unwrap();
        let commit_id = commit.id().to_string();

        if workspace.index_backend.commit_exists(&commit_id).await? {
            let tree = commit.tree().context(format!(
                "failed to get tree for commit {}",
                format_commit(&commit)
            ))?;
            reference_index.read_tree(&tree).context(format!(
                "reading tree from commit {} into index",
                format_commit(&commit)
            ))?;

            break;
        }

        if commit.parent_count() > 0 {
            let parent = commit.parent(0).context(format!(
                "fetching commit parent for {}",
                format_commit(&commit)
            ))?;

            stack.push(commit);
            stack.push(parent);
        }
    }

    while !stack.is_empty() {
        let commit = stack.pop().unwrap();
        let message = String::from_utf8_lossy(commit.message_bytes());

        println!("importing commit {}: {}", commit.id(), message);
        let tree = commit.tree().context(format!(
            "failed to get tree for commit {}",
            format_commit(&commit)
        ))?;

        let mut current_index = git2::Index::new().unwrap();
        current_index.read_tree(&tree).context(format!(
            "reading tree for commit {}",
            format_commit(&commit)
        ))?;

        let diff = git_repo
            .diff_index_to_index(&reference_index, &current_index, None)
            .context(format!(
                "diffing index for commit {}",
                format_commit(&commit)
            ))?;

        import_commit_diff(workspace, &diff, git_repo)
            .await
            .context(format!("error importing commit {}", format_commit(&commit)))?;

        let commit_id = commit.id().to_string();
        println!("recording commit {}: {}", commit_id, message);

        commit_local_changes(workspace, &commit_id, &message)
            .await
            .context(format!("error recording commit {}", format_commit(&commit)))?;

        reference_index = current_index;
    }

    Ok(())
}

async fn import_branch(
    workspace: &Workspace,
    git_repo: &git2::Repository,
    branch: &git2::Branch<'_>,
) -> Result<()> {
    let branch_name = branch.name().unwrap().unwrap();
    println!("importing branch {}", branch_name);

    match workspace
        .index_backend
        .find_branch(branch_name)
        .await
        .context(format!("error reading local branch {}", branch_name))?
    {
        Some(_branch) => {
            println!("branch already exists");
        }
        None => {
            panic!("branch creation not supported");
        }
    }

    let commit = branch
        .get()
        .peel_to_commit()
        .context("branch reference is not a commit")?;

    import_commit_sequence(workspace, git_repo, &commit).await
}

#[span_fn]
pub async fn import_git_branch_command(
    git_root_path: impl AsRef<Path>,
    branch_name: &str,
) -> Result<()> {
    let workspace = Workspace::find_in_current_directory().await?;
    let git_repo = git2::Repository::open(git_root_path.as_ref()).context(format!(
        "failed to open git repo at {}",
        git_root_path.as_ref().display()
    ))?;

    println!("git repository state: {:?}", git_repo.state());

    let git_branch = git_repo
        .find_branch(branch_name, git2::BranchType::Local)
        .context(format!("error finding branch {}", branch_name))?;

    import_branch(&workspace, &git_repo, &git_branch).await
}
