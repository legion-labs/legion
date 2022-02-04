use lgn_source_control::{CanonicalPath, Change, ChangeType, FileInfo};

macro_rules! init_test_workspace_and_index {
    () => {{
        let index_root = tempfile::tempdir().expect("failed to create temp dir");

        let index = Index::new(
            index_root
                .path()
                .to_str()
                .expect("failed to convert index_root"),
        )
        .unwrap();

        // Create the index.
        index.create().await.expect("failed to create index");

        let workspace_root = tempfile::tempdir().expect("failed to create temp dir");

        // Initialize the workspace.
        let config = WorkspaceConfig::new(
            index_root.path().display().to_string(),
            WorkspaceRegistration::new_with_current_user(),
        );

        let workspace = Workspace::init(&workspace_root.path(), config)
            .await
            .expect("failed to initialize workspace");

        (index, workspace, [index_root, workspace_root])
    }};
}

macro_rules! cleanup_test_workspace_and_index {
    ($workspace:expr, $index:expr) => {{
        // On Windows SQLite doesn't support deleting a directory with open
        // files.

        // Destroy the index.
        #[cfg(not(target_os = "windows"))]
        $index.destroy().await.unwrap();

        // Destroy the workspace.
        #[cfg(not(target_os = "windows"))]
        tokio::fs::remove_dir_all($workspace.root())
            .await
            .expect("failed to remove workspace");
    }};
}

macro_rules! create_file {
    ($workspace:expr, $path:expr, $content:literal) => {{
        let file_path = $workspace.root().join($path);

        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_other_err(format!(
                    "failed to create parent directories for file `{}`",
                    $path
                ))
                .expect("failed to create parent directories");
        }

        tokio::fs::write(file_path, $content)
            .await
            .map_other_err(format!("failed to write file `{}`", $path))
            .expect("failed to write file");
    }};
}

macro_rules! create_dir {
    ($workspace:expr, $path:expr) => {{
        let dir_path = $workspace.root().join($path);

        tokio::fs::create_dir_all(dir_path)
            .await
            .map_other_err(format!("failed to create directory `{}`", $path))
            .expect("failed to create directory");
    }};
}

macro_rules! update_file {
    ($workspace:expr, $path:expr, $content:literal) => {{
        let file_path = $workspace.root().join($path);

        tokio::fs::write(file_path, $content)
            .await
            .map_other_err(format!("failed to write file `{}`", $path))
            .expect("failed to write file");
    }};
}

macro_rules! delete_file {
    ($workspace:expr, $path:expr) => {{
        let file_path = $workspace.root().join($path);

        tokio::fs::remove_file(file_path)
            .await
            .map_other_err(format!("failed to remove file `{}`", $path))
            .expect("failed to remove file");
    }};
}

macro_rules! workspace_add_files {
    ($workspace:expr, $paths:expr) => {{
        $workspace
            .add_files($paths.into_iter().map(Path::new))
            .await
            .expect("failed to add files")
    }};
}

macro_rules! workspace_edit_files {
    ($workspace:expr, $paths:expr) => {{
        $workspace
            .edit_files($paths.into_iter().map(Path::new))
            .await
            .expect("failed to edit files")
    }};
}

macro_rules! workspace_delete_files {
    ($workspace:expr, $paths:expr) => {{
        $workspace
            .delete_files($paths.into_iter().map(Path::new))
            .await
            .expect("failed to delete files")
    }};
}

macro_rules! workspace_revert_files {
    ($workspace:expr, $paths:expr, $staging:expr) => {{
        $workspace
            .revert_files($paths.into_iter().map(Path::new), $staging)
            .await
            .expect("failed to revert files")
    }};
}

macro_rules! workspace_commit {
    ($workspace:expr, $message:literal) => {{
        $workspace.commit($message).await.expect("failed to commit")
    }};
}

macro_rules! workspace_commit_error {
    ($workspace:expr, $message:literal) => {{
        match $workspace.commit($message).await {
            Err(err) => err,
            Ok(_) => {
                panic!("commit should have failed");
            }
        }
    }};
    ($workspace:expr, $message:literal, $($err:tt)+) => {{
        match $workspace.commit($message).await {
            Err($($err)+) => {},
            Err(err) => {
                panic!("unexpected error `{}`", err);
            }
            Ok(_) => {
                panic!("commit should have failed");
            }
        }
    }};
}

pub(crate) fn cp(s: &str) -> CanonicalPath {
    CanonicalPath::new(s).unwrap()
}

pub(crate) fn add(s: &str, new_hash: &str, new_size: u64) -> Change {
    Change::new(
        cp(s),
        ChangeType::Add {
            new_info: FileInfo {
                hash: new_hash.to_owned(),
                size: new_size,
            },
        },
    )
}

pub(crate) fn edit(
    s: &str,
    old_hash: &str,
    new_hash: &str,
    old_size: u64,
    new_size: u64,
) -> Change {
    Change::new(
        cp(s),
        ChangeType::Edit {
            old_info: FileInfo {
                hash: old_hash.to_owned(),
                size: old_size,
            },
            new_info: FileInfo {
                hash: new_hash.to_owned(),
                size: new_size,
            },
        },
    )
}

pub(crate) fn delete(s: &str, old_hash: &str, old_size: u64) -> Change {
    Change::new(
        cp(s),
        ChangeType::Delete {
            old_info: FileInfo {
                hash: old_hash.to_owned(),
                size: old_size,
            },
        },
    )
}

pub(crate) use {
    cleanup_test_workspace_and_index, create_dir, create_file, delete_file,
    init_test_workspace_and_index, update_file, workspace_add_files, workspace_commit,
    workspace_commit_error, workspace_delete_files, workspace_edit_files, workspace_revert_files,
};
