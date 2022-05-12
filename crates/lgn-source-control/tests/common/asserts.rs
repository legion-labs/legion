macro_rules! assert_staged_changes {
    ($workspace:expr, $expected_changes:expr) => {{
        let changes = $workspace
            .get_staged_changes()
            .await
            .expect("failed to get staged changes")
            .into_values()
            .collect::<Vec<_>>();

        assert_eq!(changes, $expected_changes);
    }};
}

macro_rules! assert_unstaged_changes {
    ($workspace:expr, $expected_changes:expr) => {{
        let changes = $workspace
            .get_unstaged_changes()
            .await
            .expect("failed to get unstaged changes")
            .into_values()
            .collect::<Vec<_>>();

        assert_eq!(changes, $expected_changes);
    }};
}

macro_rules! assert_file_read_only {
    ($workspace:expr, $path:expr) => {{
        let file_path = $workspace.root().join($path);
        let metadata = tokio::fs::metadata(&file_path)
            .await
            .map_other_err(format!(
                "failed to get metadata for {}",
                file_path.display()
            ))
            .expect("failed to get metadata");

        let permissions = metadata.permissions();

        assert!(
            permissions.readonly(),
            "expected file {} to be read only",
            $path,
        );
    }};
}

macro_rules! assert_file_read_write {
    ($workspace:expr, $path:expr) => {{
        let file_path = $workspace.root().join($path);
        let metadata = tokio::fs::metadata(&file_path)
            .await
            .map_other_err(format!(
                "failed to get metadata for {}",
                file_path.display()
            ))
            .expect("failed to get metadata");

        let permissions = metadata.permissions();

        assert!(
            !permissions.readonly(),
            "expected file {} to be read-write",
            $path,
        );
    }};
}

macro_rules! assert_path_doesnt_exist {
    ($workspace:expr, $path:expr) => {{
        let file_path = $workspace.root().join($path);

        match tokio::fs::metadata(&file_path).await {
            Ok(_) => panic!("file `{}` should not exist", $path),
            Err(e) => {
                assert!(
                    !(e.kind() != std::io::ErrorKind::NotFound),
                    "unexpected error: {}",
                    e
                );
            }
        };
    }};
}

macro_rules! assert_file_content {
    ($workspace:expr, $path:expr, $expected_content:expr) => {{
        let file_path = $workspace.root().join($path);

        let content = tokio::fs::read_to_string(file_path)
            .await
            .map_other_err(format!("failed to read file `{}`", $path))
            .unwrap();

        assert_eq!(content, $expected_content);
    }};
}

macro_rules! assert_workspace_current_branch {
    ($workspace:expr, $branch:expr) => {{
        let current_branch = $workspace.get_current_branch().await.unwrap();

        assert_eq!(current_branch, $branch);
    }};
}

pub(crate) use {
    assert_file_content, assert_file_read_only, assert_file_read_write, assert_path_doesnt_exist,
    assert_staged_changes, assert_unstaged_changes, assert_workspace_current_branch,
};
