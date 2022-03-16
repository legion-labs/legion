use lgn_content_store2::{ChunkIdentifier, Chunker, ContentProvider};
use lgn_source_control::{CanonicalPath, Change, ChangeType};

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

        let content_store_provider = Chunker::new(SmallContentProvider::new(MemoryProvider::new()));

        let workspace = Workspace::init(&workspace_root.path(), config, &content_store_provider)
            .await
            .expect("failed to initialize workspace");

        (
            index,
            workspace,
            content_store_provider,
            [index_root, workspace_root],
        )
    }};
}

macro_rules! cleanup_test_workspace_and_index {
    ($workspace:expr, $index:expr) => {{
        // On Windows SQLite doesn't support deleting a directory with open
        // files.

        // Destroy the index.
        if cfg!(not(target_os = "windows")) {
            $index.destroy().await.unwrap();
        }

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
    ($workspace:expr, $csp:expr, $paths:expr) => {{
        $workspace
            .add_files($csp, $paths.into_iter().map(Path::new))
            .await
            .expect("failed to add files")
    }};
}

macro_rules! workspace_checkout_files {
    ($workspace:expr, $csp:expr, $paths:expr) => {{
        $workspace
            .checkout_files($csp, $paths.into_iter().map(Path::new))
            .await
            .expect("failed to edit files")
    }};
}

macro_rules! workspace_delete_files {
    ($workspace:expr, $csp:expr, $paths:expr) => {{
        $workspace
            .delete_files($csp, $paths.into_iter().map(Path::new))
            .await
            .expect("failed to delete files")
    }};
}

macro_rules! workspace_revert_files {
    ($workspace:expr, $csp:expr, $paths:expr, $staging:expr) => {{
        $workspace
            .revert_files($csp, $paths.into_iter().map(Path::new), $staging)
            .await
            .expect("failed to revert files")
    }};
}

macro_rules! workspace_commit {
    ($workspace:expr, $csp:expr, $message:literal) => {{
        $workspace
            .commit($csp, $message, lgn_source_control::CommitMode::Strict)
            .await
            .expect("failed to commit")
    }};
}

macro_rules! workspace_commit_lenient {
    ($workspace:expr, $csp:expr, $message:literal) => {{
        $workspace
            .commit($csp, $message, lgn_source_control::CommitMode::Lenient)
            .await
            .expect("failed to commit")
    }};
}

macro_rules! workspace_commit_error {
    ($workspace:expr, $csp:expr, $message:literal) => {{
        match $workspace.commit($csp, $message, lgn_source_control::CommitMode::Strict).await {
            Err(err) => err,
            Ok(_) => {
                panic!("commit should have failed");
            }
        }
    }};
    ($workspace:expr, $csp:expr, $message:literal, $($err:tt)+) => {{
        match $workspace.commit($csp, $message, lgn_source_control::CommitMode::Strict).await {
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

macro_rules! workspace_get_current_branch {
    ($workspace:expr) => {{
        $workspace
            .get_current_branch()
            .await
            .expect("failed to get current branch")
    }};
}

macro_rules! workspace_create_branch {
    ($workspace:expr, $branch_name:literal) => {{
        $workspace
            .create_branch($branch_name)
            .await
            .expect("failed to create branch")
    }};
}

macro_rules! workspace_switch_branch {
    ($workspace:expr, $csp:expr, $branch_name:literal) => {{
        $workspace
            .switch_branch($csp, $branch_name)
            .await
            .expect("failed to switch branch")
    }};
}

pub(crate) fn cp(s: &str) -> CanonicalPath {
    CanonicalPath::new(s).unwrap()
}

pub(crate) fn id(csp: &Chunker<impl ContentProvider + Send + Sync>, data: &str) -> ChunkIdentifier {
    csp.get_chunk_identifier(data.as_bytes())
}

pub(crate) fn add(s: &str, new_chunk_id: ChunkIdentifier) -> Change {
    Change::new(cp(s), ChangeType::Add { new_chunk_id })
}

pub(crate) fn edit(
    s: &str,
    old_chunk_id: ChunkIdentifier,
    new_chunk_id: ChunkIdentifier,
) -> Change {
    Change::new(
        cp(s),
        ChangeType::Edit {
            old_chunk_id,
            new_chunk_id,
        },
    )
}

pub(crate) fn delete(s: &str, old_chunk_id: ChunkIdentifier) -> Change {
    Change::new(cp(s), ChangeType::Delete { old_chunk_id })
}

pub(crate) use {
    cleanup_test_workspace_and_index, create_dir, create_file, delete_file,
    init_test_workspace_and_index, update_file, workspace_add_files, workspace_checkout_files,
    workspace_commit, workspace_commit_error, workspace_commit_lenient, workspace_create_branch,
    workspace_delete_files, workspace_get_current_branch, workspace_revert_files,
    workspace_switch_branch,
};
