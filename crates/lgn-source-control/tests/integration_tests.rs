use std::path::Path;

use lgn_source_control::{
    CanonicalPath, Change, ChangeType, Error, Index, MapOtherError, Workspace, WorkspaceConfig,
    WorkspaceRegistration,
};

async fn create_file(workspace: &Workspace, path: &str, content: &str) {
    let file_path = workspace.root().join(path);

    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_other_err(format!(
                "failed to create parent directories for file `{}`",
                path
            ))
            .unwrap();
    }

    tokio::fs::write(file_path, content)
        .await
        .map_other_err(format!("failed to write file `{}`", path))
        .unwrap();
}

async fn update_file(workspace: &Workspace, path: &str, content: &str) {
    let file_path = workspace.root().join(path);

    tokio::fs::write(file_path, content)
        .await
        .map_other_err(format!("failed to write file `{}`", path))
        .unwrap();
}

async fn assert_file_read_only(workspace: &Workspace, path: &str, readonly: bool) {
    let file_path = workspace.root().join(path);
    let metadata = tokio::fs::metadata(&file_path)
        .await
        .map_other_err(format!(
            "failed to get metadata for {}",
            file_path.display()
        ))
        .unwrap();

    let permissions = metadata.permissions();

    assert_eq!(
        permissions.readonly(),
        readonly,
        "expected file {} readonly status to be {}",
        path,
        readonly
    );
}

async fn assert_path_doesnt_exist(workspace: &Workspace, path: &str) {
    let file_path = workspace.root().join(path);

    match tokio::fs::metadata(&file_path).await {
        Ok(_) => panic!("file `{}` should not exist", path),
        Err(e) => {
            assert!(
                !(e.kind() != std::io::ErrorKind::NotFound),
                "unexpected error: {}",
                e
            );
        }
    };
}

async fn assert_file_content(workspace: &Workspace, path: &str, expected_content: &str) {
    let file_path = workspace.root().join(path);

    let content = tokio::fs::read_to_string(file_path)
        .await
        .map_other_err(format!("failed to read file `{}`", path))
        .unwrap();

    assert_eq!(content, expected_content);
}

fn cp(s: &str) -> CanonicalPath {
    CanonicalPath::new(s).unwrap()
}

fn add(s: &str, new_hash: &str) -> Change {
    Change::new(
        cp(s),
        ChangeType::Add {
            new_hash: new_hash.to_owned(),
        },
    )
}

fn edit(s: &str, old_hash: &str, new_hash: &str) -> Change {
    Change::new(
        cp(s),
        ChangeType::Edit {
            old_hash: old_hash.to_owned(),
            new_hash: new_hash.to_owned(),
        },
    )
}

fn delete(s: &str, old_hash: &str) -> Change {
    Change::new(
        cp(s),
        ChangeType::Delete {
            old_hash: old_hash.to_owned(),
        },
    )
}

async fn assert_staged_changes(workspace: &Workspace, expected_changes: &[Change]) {
    let staged_changes = workspace
        .get_staged_changes()
        .await
        .unwrap()
        .into_values()
        .collect::<Vec<_>>();

    assert_eq!(staged_changes, expected_changes);
}

async fn assert_unstaged_changes(workspace: &Workspace, expected_changes: &[Change]) {
    let staged_changes = workspace
        .get_unstaged_changes()
        .await
        .unwrap()
        .into_values()
        .collect::<Vec<_>>();

    assert_eq!(staged_changes, expected_changes);
}

#[tokio::test]
async fn test_full_flow() {
    //let root = &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/integration");
    let index_root = tempfile::tempdir().unwrap();

    let index = Index::new(index_root.path().to_str().unwrap()).unwrap();

    // Create the index.
    index.create().await.unwrap();

    let workspace_root = tempfile::tempdir().unwrap();

    // Initialize the workspace.
    let config = WorkspaceConfig::new(
        index_root.path().display().to_string(),
        WorkspaceRegistration::new_with_current_user(),
    );
    let workspace = Workspace::init(&workspace_root.path(), config)
        .await
        .unwrap();

    // Add some files.
    create_file(&workspace, "apple.txt", "I am an apple").await;
    create_file(&workspace, "orange.txt", "I am an orange").await;
    create_file(&workspace, "vegetables/carrot.txt", "I am a carrot").await;

    assert_unstaged_changes(
        &workspace,
        &[
            add(
                "/apple.txt",
                "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
            ),
            add(
                "/orange.txt",
                "9bd4bfdc816b05ebc6fa07ddb99991e65097f22d53b3e492dadebef90f25baa0",
            ),
            add(
                "/vegetables/carrot.txt",
                "3cfa2f8506a5d2e1a397a03f1e92f5d96e77315d5d428568848e100d14089ce9",
            ),
        ],
    )
    .await;
    assert_staged_changes(&workspace, &[]).await;

    let new_added_files = workspace.add_files([Path::new(".")]).await.unwrap();

    assert_eq!(
        new_added_files,
        [
            cp("/apple.txt"),
            cp("/orange.txt"),
            cp("/vegetables/carrot.txt")
        ]
        .into(),
    );

    assert_unstaged_changes(&workspace, &[]).await;
    assert_staged_changes(
        &workspace,
        &[
            add(
                "/apple.txt",
                "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
            ),
            add(
                "/orange.txt",
                "9bd4bfdc816b05ebc6fa07ddb99991e65097f22d53b3e492dadebef90f25baa0",
            ),
            add(
                "/vegetables/carrot.txt",
                "3cfa2f8506a5d2e1a397a03f1e92f5d96e77315d5d428568848e100d14089ce9",
            ),
        ],
    )
    .await;

    // Re-adding the same files should be a no-op.
    let new_added_files = workspace.add_files([Path::new(".")]).await.unwrap();

    assert_eq!(new_added_files, [].into());

    assert_staged_changes(
        &workspace,
        &[
            add(
                "/apple.txt",
                "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
            ),
            add(
                "/orange.txt",
                "9bd4bfdc816b05ebc6fa07ddb99991e65097f22d53b3e492dadebef90f25baa0",
            ),
            add(
                "/vegetables/carrot.txt",
                "3cfa2f8506a5d2e1a397a03f1e92f5d96e77315d5d428568848e100d14089ce9",
            ),
        ],
    )
    .await;

    // Editing the same files should be a no-op.
    let new_edited_files = workspace.edit_files([Path::new(".")]).await.unwrap();

    assert_eq!(new_edited_files, [].into());

    assert_staged_changes(
        &workspace,
        &[
            add(
                "/apple.txt",
                "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
            ),
            add(
                "/orange.txt",
                "9bd4bfdc816b05ebc6fa07ddb99991e65097f22d53b3e492dadebef90f25baa0",
            ),
            add(
                "/vegetables/carrot.txt",
                "3cfa2f8506a5d2e1a397a03f1e92f5d96e77315d5d428568848e100d14089ce9",
            ),
        ],
    )
    .await;

    // Commit the files.
    workspace.commit("Added some fruits").await.unwrap();

    assert_staged_changes(&workspace, &[]).await;
    assert_file_read_only(&workspace, "apple.txt", true).await;
    assert_file_content(&workspace, "apple.txt", "I am an apple").await;
    assert_file_read_only(&workspace, "orange.txt", true).await;
    assert_file_content(&workspace, "orange.txt", "I am an orange").await;
    assert_file_read_only(&workspace, "vegetables/carrot.txt", true).await;
    assert_file_content(&workspace, "vegetables/carrot.txt", "I am a carrot").await;

    // Let's now edit one file.
    let new_edited_files = workspace
        .edit_files([Path::new("vegetables")])
        .await
        .unwrap();

    assert_eq!(new_edited_files, [cp("/vegetables/carrot.txt")].into());

    assert_unstaged_changes(&workspace, &[]).await;
    assert_staged_changes(
        &workspace,
        &[edit(
            "/vegetables/carrot.txt",
            "3cfa2f8506a5d2e1a397a03f1e92f5d96e77315d5d428568848e100d14089ce9",
            "3cfa2f8506a5d2e1a397a03f1e92f5d96e77315d5d428568848e100d14089ce9",
        )],
    )
    .await;

    assert_file_read_only(&workspace, "vegetables/carrot.txt", false).await;
    assert_file_content(&workspace, "vegetables/carrot.txt", "I am a carrot").await;

    update_file(&workspace, "vegetables/carrot.txt", "I am a new carrot").await;

    assert_unstaged_changes(
        &workspace,
        &[edit(
            "/vegetables/carrot.txt",
            "3cfa2f8506a5d2e1a397a03f1e92f5d96e77315d5d428568848e100d14089ce9",
            "d041184eda20d2bd0dd05f2fbe96134d1832697ba9cb97d137df76fb6231424c",
        )],
    )
    .await;

    // Commit the change should fail, as we have one edited file whose hash
    // changed but the change was not staged.
    match workspace.commit("Edited the carrot").await {
        Err(Error::EmptyCommitNotAllowed) => {}
        Err(err) => {
            panic!("unexpected error: {:?}", err);
        }
        Ok(_) => {
            panic!("commit should have failed");
        }
    }

    // Add or edit should work the same here: let's first try with add. We'll test with edit later.
    let new_added_files = workspace.add_files([Path::new(".")]).await.unwrap();

    assert_eq!(new_added_files, [cp("/vegetables/carrot.txt")].into());

    assert_staged_changes(
        &workspace,
        &[edit(
            "/vegetables/carrot.txt",
            "3cfa2f8506a5d2e1a397a03f1e92f5d96e77315d5d428568848e100d14089ce9",
            "d041184eda20d2bd0dd05f2fbe96134d1832697ba9cb97d137df76fb6231424c",
        )],
    )
    .await;

    // Commit the files.
    workspace.commit("Edited the carrot").await.unwrap();

    assert_staged_changes(&workspace, &[]).await;
    assert_file_read_only(&workspace, "vegetables/carrot.txt", true).await;
    assert_file_content(&workspace, "vegetables/carrot.txt", "I am a new carrot").await;
    assert_file_content(&workspace, "apple.txt", "I am an apple").await;

    // Let's delete a file.
    let new_deleted_files = workspace
        .delete_files([Path::new("vegetables/carrot.txt")])
        .await
        .unwrap();

    assert_eq!(new_deleted_files, [cp("/vegetables/carrot.txt")].into());

    assert_staged_changes(
        &workspace,
        &[delete(
            "/vegetables/carrot.txt",
            "d041184eda20d2bd0dd05f2fbe96134d1832697ba9cb97d137df76fb6231424c",
        )],
    )
    .await;

    assert_path_doesnt_exist(&workspace, "vegetables/carrot.txt").await;
    assert_file_content(&workspace, "apple.txt", "I am an apple").await;

    // Commit the files.
    workspace.commit("Removed the carrot").await.unwrap();

    assert_staged_changes(&workspace, &[]).await;
    assert_path_doesnt_exist(&workspace, "vegetables/carrot.txt").await;

    create_file(&workspace, "banana.txt", "I am a banana").await;

    let new_added_files = workspace.add_files([Path::new(".")]).await.unwrap();

    assert_eq!(new_added_files, [cp("/banana.txt")].into());

    let new_deleted_files = workspace
        .delete_files([Path::new("banana.txt")])
        .await
        .unwrap();

    assert_eq!(new_deleted_files, [cp("/banana.txt")].into());
    assert_path_doesnt_exist(&workspace, "banana.txt").await;

    // The file was not really in the tree: it was just staged for addition. So its removal is actually a no-op.
    assert_staged_changes(&workspace, &[]).await;

    // Adding an empty directory but existing should yield no error and add no files.
    let new_added_files = workspace
        .add_files([Path::new("vegetables")])
        .await
        .unwrap();

    assert_eq!(new_added_files, [].into(),);

    // Adding an non-existing path should fail.
    match workspace.add_files([Path::new("non/existing/path")]).await {
        Err(Error::UnmatchedPath { .. }) => {}
        Err(err) => {
            panic!("unexpected error: {:?}", err);
        }
        Ok(_) => {
            panic!("add should have failed");
        }
    };

    let new_edited_files = workspace
        .edit_files([Path::new("apple.txt")])
        .await
        .unwrap();

    assert_eq!(new_edited_files, [cp("/apple.txt")].into());

    assert_staged_changes(
        &workspace,
        &[edit(
            "/apple.txt",
            "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
            "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
        )],
    )
    .await;

    assert_file_read_only(&workspace, "apple.txt", false).await;
    assert_file_content(&workspace, "apple.txt", "I am an apple").await;
    update_file(&workspace, "apple.txt", "I am a new apple").await;

    // The recent change was not staged and thus should not be listed.
    assert_staged_changes(
        &workspace,
        &[edit(
            "/apple.txt",
            "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
            "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
        )],
    )
    .await;

    // Stage the change, this time using `edit`. Using `add` would have worked as well.
    let new_edited_files = workspace
        .edit_files([Path::new("apple.txt")])
        .await
        .unwrap();

    assert_eq!(new_edited_files, [cp("/apple.txt")].into());

    assert_staged_changes(
        &workspace,
        &[edit(
            "/apple.txt",
            "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
            "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
        )],
    )
    .await;

    // Let's make another change but do not stage it.
    update_file(&workspace, "apple.txt", "I am an even newer apple").await;

    // Commit the files.
    workspace.commit("Update the apple").await.unwrap();

    // File should not be read only nor unlocked as it has unstaged changes.
    assert_file_read_only(&workspace, "apple.txt", false).await;
    assert_file_content(&workspace, "apple.txt", "I am an even newer apple").await;

    // We should still have some unstaged changes as we did not stage them yet.
    assert_staged_changes(&workspace, &[]).await;
    assert_unstaged_changes(
        &workspace,
        &[edit(
            "/apple.txt",
            "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
            "3ff8d72c2de3847451c2430776f2f9e38abf0e7eae5aabcdc4dfae91dacadc51",
        )],
    )
    .await;

    // Reverting the file with unstaged changes should work.
    let new_reverted_files = workspace
        .revert_files([Path::new("apple.txt")])
        .await
        .unwrap();

    assert_eq!(new_reverted_files, [cp("/apple.txt")].into());

    assert_staged_changes(&workspace, &[]).await;
    assert_unstaged_changes(&workspace, &[]).await;

    assert_file_read_only(&workspace, "apple.txt", true).await;
    assert_file_content(&workspace, "apple.txt", "I am a new apple").await;

    // Let's make a change but stage it this time.
    let new_edited_files = workspace
        .edit_files([Path::new("apple.txt")])
        .await
        .unwrap();

    assert_eq!(new_edited_files, [cp("/apple.txt"),].into());

    update_file(&workspace, "apple.txt", "I am an even newer apple").await;
    create_file(&workspace, "strawberry.txt", "I am a strawberry").await;

    let new_added_files = workspace.add_files([Path::new(".")]).await.unwrap();

    assert_eq!(
        new_added_files,
        [cp("/apple.txt"), cp("/strawberry.txt")].into()
    );

    assert_staged_changes(
        &workspace,
        &[
            edit(
                "/apple.txt",
                "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
                "3ff8d72c2de3847451c2430776f2f9e38abf0e7eae5aabcdc4dfae91dacadc51",
            ),
            add(
                "/strawberry.txt",
                "ee15463ad8b18f1bec0374e55969913f81c205e02cc9fb331e8ca60211344ee2",
            ),
        ],
    )
    .await;
    assert_unstaged_changes(&workspace, &[]).await;

    // Reverting the file with staged changes should work too.
    let new_reverted_files = workspace.revert_files([Path::new(".")]).await.unwrap();

    assert_eq!(
        new_reverted_files,
        [cp("/apple.txt"), cp("/strawberry.txt")].into()
    );

    // Untracked files that are reverted for add are not deleted.
    assert_staged_changes(&workspace, &[]).await;
    assert_unstaged_changes(
        &workspace,
        &[add(
            "/strawberry.txt",
            "ee15463ad8b18f1bec0374e55969913f81c205e02cc9fb331e8ca60211344ee2",
        )],
    )
    .await;

    assert_file_read_only(&workspace, "apple.txt", true).await;
    assert_file_content(&workspace, "apple.txt", "I am a new apple").await;
    assert_file_content(&workspace, "strawberry.txt", "I am a strawberry").await;

    // Destroy the index.
    #[cfg(not(target_os = "windows"))]
    index.destroy().await.unwrap();
}
