mod common;

#[allow(clippy::wildcard_imports)]
use common::*;

use std::path::Path;

use lgn_source_control::{
    Error, Index, MapOtherError, Staging, Workspace, WorkspaceConfig, WorkspaceRegistration,
};

#[tokio::test]
async fn test_add_and_commit() {
    let (index, ws, _paths) = init_test_workspace_and_index!();

    // Add some files.
    create_file!(ws, "apple.txt", "I am an apple");
    create_file!(ws, "orange.txt", "I am an orange");
    create_file!(ws, "vegetables/carrot.txt", "I am a carrot");

    assert_unstaged_changes!(
        ws,
        [
            add(
                "/apple.txt",
                "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
                13,
            ),
            add(
                "/orange.txt",
                "9bd4bfdc816b05ebc6fa07ddb99991e65097f22d53b3e492dadebef90f25baa0",
                14,
            ),
            add(
                "/vegetables/carrot.txt",
                "3cfa2f8506a5d2e1a397a03f1e92f5d96e77315d5d428568848e100d14089ce9",
                13,
            ),
        ]
    );
    assert_staged_changes!(ws, []);

    let new_added_files = workspace_add_files!(ws, ["."]);

    assert_eq!(
        new_added_files,
        [
            cp("/apple.txt"),
            cp("/orange.txt"),
            cp("/vegetables/carrot.txt")
        ]
        .into(),
    );

    assert_unstaged_changes!(ws, []);
    assert_staged_changes!(
        ws,
        [
            add(
                "/apple.txt",
                "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
                13,
            ),
            add(
                "/orange.txt",
                "9bd4bfdc816b05ebc6fa07ddb99991e65097f22d53b3e492dadebef90f25baa0",
                14,
            ),
            add(
                "/vegetables/carrot.txt",
                "3cfa2f8506a5d2e1a397a03f1e92f5d96e77315d5d428568848e100d14089ce9",
                13,
            ),
        ]
    );

    // Re-adding the same files should be a no-op.
    let new_added_files = workspace_add_files!(ws, ["."]);

    assert_eq!(new_added_files, [].into());

    assert_staged_changes!(
        ws,
        [
            add(
                "/apple.txt",
                "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
                13,
            ),
            add(
                "/orange.txt",
                "9bd4bfdc816b05ebc6fa07ddb99991e65097f22d53b3e492dadebef90f25baa0",
                14,
            ),
            add(
                "/vegetables/carrot.txt",
                "3cfa2f8506a5d2e1a397a03f1e92f5d96e77315d5d428568848e100d14089ce9",
                13,
            ),
        ]
    );

    // Editing the same files should be a no-op.
    let new_edited_files = workspace_edit_files!(ws, ["."]);

    assert_eq!(new_edited_files, [].into());

    assert_staged_changes!(
        ws,
        [
            add(
                "/apple.txt",
                "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
                13,
            ),
            add(
                "/orange.txt",
                "9bd4bfdc816b05ebc6fa07ddb99991e65097f22d53b3e492dadebef90f25baa0",
                14,
            ),
            add(
                "/vegetables/carrot.txt",
                "3cfa2f8506a5d2e1a397a03f1e92f5d96e77315d5d428568848e100d14089ce9",
                13,
            ),
        ]
    );

    // Commit the files.
    workspace_commit!(ws, "Added some fruits");

    assert_staged_changes!(ws, []);
    assert_file_read_only!(ws, "apple.txt");
    assert_file_content!(ws, "apple.txt", "I am an apple");
    assert_file_read_only!(ws, "orange.txt");
    assert_file_content!(ws, "orange.txt", "I am an orange");
    assert_file_read_only!(ws, "vegetables/carrot.txt");
    assert_file_content!(ws, "vegetables/carrot.txt", "I am a carrot");

    // Committing the change should fail, as the commit is empty.
    workspace_commit_error!(ws, "Edited the carrot", Error::EmptyCommitNotAllowed);

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_edit_and_commit() {
    let (index, ws, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am an apple");
    create_file!(ws, "orange.txt", "I am an orange");
    create_file!(ws, "vegetables/carrot.txt", "I am a carrot");
    workspace_add_files!(ws, ["."]);
    workspace_commit!(ws, "Added some fruits");

    // Let's now edit one file.
    let new_edited_files = ws.edit_files([Path::new("vegetables")]).await.unwrap();

    assert_eq!(new_edited_files, [cp("/vegetables/carrot.txt")].into());

    assert_unstaged_changes!(ws, []);
    assert_staged_changes!(
        ws,
        [edit(
            "/vegetables/carrot.txt",
            "3cfa2f8506a5d2e1a397a03f1e92f5d96e77315d5d428568848e100d14089ce9",
            "3cfa2f8506a5d2e1a397a03f1e92f5d96e77315d5d428568848e100d14089ce9",
            13,
            13,
        )]
    );

    assert_file_read_write!(ws, "vegetables/carrot.txt");
    assert_file_content!(ws, "vegetables/carrot.txt", "I am a carrot");

    update_file!(ws, "vegetables/carrot.txt", "I am a new carrot");

    assert_unstaged_changes!(
        ws,
        [edit(
            "/vegetables/carrot.txt",
            "3cfa2f8506a5d2e1a397a03f1e92f5d96e77315d5d428568848e100d14089ce9",
            "d041184eda20d2bd0dd05f2fbe96134d1832697ba9cb97d137df76fb6231424c",
            13,
            17,
        )]
    );

    // Committing the change should fail, as we have one edited file whose hash
    // changed but the change was not staged.
    match workspace_commit_error!(ws, "Edited the carrot") {
        Error::UnchangedFilesMarkedForEdition { paths } => {
            assert_eq!(paths, [cp("/vegetables/carrot.txt")].into());
        }
        e => panic!("Unexpected error: {:?}", e),
    };

    // Add or edit should work the same here: let's first try with add. We'll test with edit later.
    let new_added_files = workspace_add_files!(ws, ["."]);

    assert_eq!(new_added_files, [cp("/vegetables/carrot.txt")].into());

    assert_staged_changes!(
        ws,
        [edit(
            "/vegetables/carrot.txt",
            "3cfa2f8506a5d2e1a397a03f1e92f5d96e77315d5d428568848e100d14089ce9",
            "d041184eda20d2bd0dd05f2fbe96134d1832697ba9cb97d137df76fb6231424c",
            13,
            17,
        )]
    );

    // Commit the files.
    workspace_commit!(ws, "Edited the carrot");

    assert_staged_changes!(ws, []);
    assert_file_read_only!(ws, "vegetables/carrot.txt");
    assert_file_content!(ws, "vegetables/carrot.txt", "I am a new carrot");
    assert_file_content!(ws, "apple.txt", "I am an apple");

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_delete_and_commit() {
    let (index, ws, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am an apple");
    create_file!(ws, "orange.txt", "I am an orange");
    create_file!(ws, "vegetables/carrot.txt", "I am a new carrot");
    workspace_add_files!(ws, ["."]);
    workspace_commit!(ws, "Added some fruits");

    // Let's delete a file.
    let new_deleted_files = workspace_delete_files!(ws, ["vegetables/carrot.txt"]);

    assert_eq!(new_deleted_files, [cp("/vegetables/carrot.txt")].into());

    assert_staged_changes!(
        ws,
        [delete(
            "/vegetables/carrot.txt",
            "d041184eda20d2bd0dd05f2fbe96134d1832697ba9cb97d137df76fb6231424c",
            17,
        )]
    );

    assert_path_doesnt_exist!(ws, "vegetables/carrot.txt");
    assert_file_content!(ws, "apple.txt", "I am an apple");

    // Commit the files.
    workspace_commit!(ws, "Removed the carrot");

    assert_staged_changes!(ws, []);
    assert_path_doesnt_exist!(ws, "vegetables/carrot.txt");

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_add_empty_directory() {
    let (index, ws, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am an apple");
    workspace_add_files!(ws, ["."]);
    workspace_commit!(ws, "Added some fruits");

    create_dir!(ws, "vegetables");

    // Adding an empty directory but existing should yield no error and add no files.
    let new_added_files = ws.add_files([Path::new("vegetables")]).await.unwrap();

    assert_eq!(new_added_files, [].into(),);

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_add_non_existing_path() {
    let (index, ws, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am an apple");
    workspace_add_files!(ws, ["."]);
    workspace_commit!(ws, "Added some fruits");

    // Adding an non-existing path should fail.
    match ws.add_files([Path::new("non/existing/path")]).await {
        Err(Error::UnmatchedPath { .. }) => {}
        Err(err) => {
            panic!("unexpected error: {:?}", err);
        }
        Ok(_) => {
            panic!("add should have failed");
        }
    };

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_add_then_delete() {
    let (index, ws, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am an apple");
    create_file!(ws, "orange.txt", "I am an orange");
    workspace_add_files!(ws, ["."]);
    workspace_commit!(ws, "Added some fruits");

    create_file!(ws, "banana.txt", "I am a banana");

    let new_added_files = workspace_add_files!(ws, ["."]);

    assert_eq!(new_added_files, [cp("/banana.txt")].into());

    let new_deleted_files = workspace_delete_files!(ws, ["banana.txt"]);

    assert_eq!(new_deleted_files, [cp("/banana.txt")].into());
    assert_path_doesnt_exist!(ws, "banana.txt");

    // The file was not really in the tree: it was just staged for addition. So its removal is actually a no-op.
    assert_unstaged_changes!(ws, []);
    assert_staged_changes!(ws, []);

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_edit_and_commit_with_extra_unstaged_changes_then_revert() {
    let (index, ws, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am an apple");
    create_file!(ws, "orange.txt", "I am an orange");
    workspace_add_files!(ws, ["."]);
    workspace_commit!(ws, "Added some fruits");

    let new_edited_files = workspace_edit_files!(ws, ["apple.txt"]);

    assert_eq!(new_edited_files, [cp("/apple.txt")].into());

    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
            "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
            13,
            13,
        )]
    );

    assert_file_read_write!(ws, "apple.txt");
    assert_file_content!(ws, "apple.txt", "I am an apple");
    update_file!(ws, "apple.txt", "I am a new apple");

    // The recent change was not staged and thus should not be listed.
    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
            "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
            13,
            13,
        )]
    );

    // Stage the change, this time using `edit`. Using `add` would have worked as well.
    let new_edited_files = workspace_edit_files!(ws, ["apple.txt"]);

    assert_eq!(new_edited_files, [cp("/apple.txt")].into());

    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            "bec0c979dad52f98fa6772fd89acfa5c93b856bdc1331d1a73694a194f121181",
            "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
            13,
            16,
        )]
    );

    // Let's make another change but do not stage it.
    update_file!(ws, "apple.txt", "I am an even newer apple");

    // Commit the files.
    workspace_commit!(ws, "Update the apple");

    // File should not be read only nor unlocked as it has unstaged changes.
    assert_file_read_write!(ws, "apple.txt");
    assert_file_content!(ws, "apple.txt", "I am an even newer apple");

    // We should still have some unstaged changes as we did not stage them yet.
    // Also the file should still be checked out for edition and locked.
    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
            "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
            16,
            16,
        )]
    );
    assert_unstaged_changes!(
        ws,
        [edit(
            "/apple.txt",
            "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
            "3ff8d72c2de3847451c2430776f2f9e38abf0e7eae5aabcdc4dfae91dacadc51",
            16,
            24,
        )]
    );

    // Reverting the file with unstaged changes should work.
    let new_reverted_files = workspace_revert_files!(ws, ["apple.txt"], Staging::StagedAndUnstaged);

    assert_eq!(new_reverted_files, [cp("/apple.txt")].into());

    assert_staged_changes!(ws, []);
    assert_unstaged_changes!(ws, []);

    assert_file_read_only!(ws, "apple.txt");
    assert_file_content!(ws, "apple.txt", "I am a new apple");

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_revert_after_add_and_edit() {
    let (index, ws, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am a new apple");
    create_file!(ws, "orange.txt", "I am an orange");
    workspace_add_files!(ws, ["."]);
    workspace_commit!(ws, "Added some fruits");

    // Let's make a change but stage it this time.
    let new_edited_files = workspace_edit_files!(ws, ["apple.txt"]);

    assert_eq!(new_edited_files, [cp("/apple.txt"),].into());

    update_file!(ws, "apple.txt", "I am an even newer apple");
    create_file!(ws, "strawberry.txt", "I am a strawberry");

    let new_added_files = workspace_add_files!(ws, ["."]);

    assert_eq!(
        new_added_files,
        [cp("/apple.txt"), cp("/strawberry.txt")].into()
    );

    assert_staged_changes!(
        ws,
        [
            edit(
                "/apple.txt",
                "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
                "3ff8d72c2de3847451c2430776f2f9e38abf0e7eae5aabcdc4dfae91dacadc51",
                16,
                24,
            ),
            add(
                "/strawberry.txt",
                "ee15463ad8b18f1bec0374e55969913f81c205e02cc9fb331e8ca60211344ee2",
                17,
            ),
        ]
    );
    assert_unstaged_changes!(ws, []);

    // Reverting the file with staged changes should work too.
    let new_reverted_files = workspace_revert_files!(ws, ["."], Staging::StagedAndUnstaged);

    assert_eq!(
        new_reverted_files,
        [cp("/apple.txt"), cp("/strawberry.txt")].into()
    );

    // Untracked files that are reverted for add are not deleted.
    assert_staged_changes!(ws, []);
    assert_unstaged_changes!(
        ws,
        [add(
            "/strawberry.txt",
            "ee15463ad8b18f1bec0374e55969913f81c205e02cc9fb331e8ca60211344ee2",
            17,
        )]
    );

    assert_file_read_only!(ws, "apple.txt");
    assert_file_read_write!(ws, "strawberry.txt");
    assert_file_content!(ws, "apple.txt", "I am a new apple");
    assert_file_content!(ws, "strawberry.txt", "I am a strawberry");

    // Let's delete the file: it should not appear unstaged anymore.
    delete_file!(ws, "strawberry.txt");

    assert_staged_changes!(ws, []);
    assert_unstaged_changes!(ws, []);

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_revert_staged_only_with_unstaged_changes() {
    let (index, ws, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am a new apple");
    create_file!(ws, "orange.txt", "I am an orange");
    workspace_add_files!(ws, ["."]);
    workspace_commit!(ws, "Added some fruits");

    // Let's mark a file for edition, change it but do not stage it. Then let's revert it in staging.
    let new_edited_files = workspace_edit_files!(ws, ["apple.txt"]);

    assert_eq!(new_edited_files, [cp("/apple.txt")].into());
    update_file!(ws, "apple.txt", "I am an even newer apple");

    assert_unstaged_changes!(
        ws,
        [edit(
            "/apple.txt",
            "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
            "3ff8d72c2de3847451c2430776f2f9e38abf0e7eae5aabcdc4dfae91dacadc51",
            16,
            24,
        )]
    );
    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
            "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
            16,
            16,
        )]
    );

    // The file has unstaged changes, so reverting in staged-only mode should
    // not affect it.
    let new_reverted_files = workspace_revert_files!(ws, ["apple.txt"], Staging::StagedOnly);

    assert_eq!(new_reverted_files, [].into());

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_revert_staged_only_with_staged_and_unstaged_changes() {
    let (index, ws, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am a new apple");
    create_file!(ws, "orange.txt", "I am an orange");
    workspace_add_files!(ws, ["."]);
    workspace_commit!(ws, "Added some fruits");

    let new_edited_files = workspace_edit_files!(ws, ["apple.txt"]);

    assert_eq!(new_edited_files, [cp("/apple.txt")].into());
    update_file!(ws, "apple.txt", "I am an even newer apple");

    let new_added_files = workspace_add_files!(ws, ["apple.txt"]);

    assert_eq!(new_added_files, [cp("/apple.txt"),].into(),);

    assert_unstaged_changes!(ws, []);
    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
            "3ff8d72c2de3847451c2430776f2f9e38abf0e7eae5aabcdc4dfae91dacadc51",
            16,
            24,
        )]
    );

    update_file!(ws, "apple.txt", "Unstaged modification");

    let new_reverted_files = workspace_revert_files!(ws, ["apple.txt"], Staging::StagedOnly);

    assert_eq!(new_reverted_files, [cp("/apple.txt")].into());

    assert_file_read_write!(ws, "apple.txt");
    assert_file_content!(ws, "apple.txt", "Unstaged modification");

    assert_unstaged_changes!(
        ws,
        [edit(
            "/apple.txt",
            "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
            "d5c062b844a8a2fce186e6552465512dd7f08fbec83f2580d72f560ab51c541e",
            16,
            21,
        )]
    );
    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
            "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
            16,
            16,
        )]
    );

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_revert_unstaged_only_with_unstaged_changes() {
    let (index, ws, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am a new apple");
    create_file!(ws, "orange.txt", "I am an orange");
    workspace_add_files!(ws, ["."]);
    workspace_commit!(ws, "Added some fruits");

    let new_edited_files = workspace_edit_files!(ws, ["apple.txt"]);

    assert_eq!(new_edited_files, [cp("/apple.txt")].into());
    update_file!(ws, "apple.txt", "Unstaged modification");

    // This time let's only revert unstaged changes.
    let new_reverted_files = workspace_revert_files!(ws, ["apple.txt"], Staging::UnstagedOnly);

    assert_eq!(new_reverted_files, [cp("/apple.txt")].into());

    assert_file_read_write!(ws, "apple.txt");
    assert_file_content!(ws, "apple.txt", "I am a new apple");

    assert_unstaged_changes!(ws, []);
    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
            "669da1806b2e40098034455c9346afe212ac4f258009eda0d2e8903c4569a35a",
            16,
            16,
        )]
    );

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_sync_forward_and_backward() {
    let (index, ws, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "apple version 1");
    create_file!(ws, "orange.txt", "orange version 1");
    workspace_add_files!(ws, ["."]);
    let commit_id_1 = workspace_commit!(ws, "version 1");

    // Update an existing file.
    workspace_edit_files!(ws, ["apple.txt"]);
    update_file!(ws, "apple.txt", "apple version 2");

    // Create a new one.
    create_file!(ws, "pear.txt", "pear version 1");
    workspace_add_files!(ws, ["apple.txt", "pear.txt"]);

    // And delete an old one.
    workspace_delete_files!(ws, ["orange.txt"]);

    let commit_id_2 = workspace_commit!(ws, "version 2");

    assert_file_content!(ws, "apple.txt", "apple version 2");
    assert_file_read_only!(ws, "apple.txt");
    assert_path_doesnt_exist!(ws, "orange.txt");
    assert_file_content!(ws, "pear.txt", "pear version 1");
    assert_file_read_only!(ws, "pear.txt");

    // Try to sync back to the previous commit.
    let current_commit_id = ws.sync_to(&commit_id_1).await.unwrap();
    assert_eq!(current_commit_id, commit_id_1);

    assert_path_doesnt_exist!(ws, "pear.txt");
    assert_file_content!(ws, "apple.txt", "apple version 1");
    assert_file_read_only!(ws, "apple.txt");
    assert_file_content!(ws, "orange.txt", "orange version 1");
    assert_file_read_only!(ws, "orange.txt");

    // Sync back to the latest commit.
    let current_commit_id = ws.sync().await.unwrap();
    assert_eq!(current_commit_id, commit_id_2);

    assert_file_content!(ws, "apple.txt", "apple version 2");
    assert_file_read_only!(ws, "apple.txt");
    assert_path_doesnt_exist!(ws, "orange.txt");
    assert_file_content!(ws, "pear.txt", "pear version 1");
    assert_file_read_only!(ws, "pear.txt");

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_sync_forward_with_non_conflicting_changes() {
    let (index, ws, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "tangerine.txt", "tangerine version 1");
    create_file!(ws, "cantaloupe.txt", "cantaloupe version 1");
    create_file!(ws, "apple.txt", "apple version 1");
    create_file!(ws, "orange.txt", "orange version 1");
    workspace_add_files!(ws, ["."]);
    let commit_id_1 = workspace_commit!(ws, "version 1");

    // Update an existing file.
    workspace_edit_files!(ws, ["apple.txt"]);
    update_file!(ws, "apple.txt", "apple version 2");

    // Create a new one.
    create_file!(ws, "pear.txt", "pear version 1");
    workspace_add_files!(ws, ["apple.txt", "pear.txt"]);

    // And delete an old one.
    workspace_delete_files!(ws, ["orange.txt"]);

    let commit_id_2 = workspace_commit!(ws, "version 2");

    // Make some staged and unstaged changes.
    workspace_edit_files!(ws, ["tangerine.txt", "cantaloupe.txt"]);
    update_file!(ws, "tangerine.txt", "tangerine version 2");
    update_file!(ws, "cantaloupe.txt", "cantaloupe version 2");
    workspace_add_files!(ws, ["tangerine.txt"]);

    assert_staged_changes!(
        ws,
        [
            edit(
                "/cantaloupe.txt",
                "51cc30f874df85b05976c169c23871ddd1a572a4a266b021bba929427e9f1d33",
                "51cc30f874df85b05976c169c23871ddd1a572a4a266b021bba929427e9f1d33",
                20,
                20
            ),
            edit(
                "/tangerine.txt",
                "c753b9ae0bc47041a91eb967401fdeb5c583f64a5ea55a7c1bf381f4634193f5",
                "e39d1f2bdd246ab1f4df9666cfa3a8297cf342f0e8c4cb0827b79c3583848369",
                19,
                19
            ),
        ]
    );
    assert_unstaged_changes!(
        ws,
        [edit(
            "/cantaloupe.txt",
            "51cc30f874df85b05976c169c23871ddd1a572a4a266b021bba929427e9f1d33",
            "d900c4bbbc5836bc7c0cab9e2d639f11324159fc0c37efbda2e7df5b64e40d4a",
            20,
            20
        )]
    );

    // Try to sync back to the previous commit: this should work even though we
    // have local changes as those do not conflict at all.
    let current_commit_id = ws.sync_to(&commit_id_1).await.unwrap();
    assert_eq!(current_commit_id, commit_id_1);

    assert_path_doesnt_exist!(ws, "pear.txt");
    assert_file_content!(ws, "apple.txt", "apple version 1");
    assert_file_read_only!(ws, "apple.txt");
    assert_file_content!(ws, "orange.txt", "orange version 1");
    assert_file_read_only!(ws, "orange.txt");
    assert_file_content!(ws, "tangerine.txt", "tangerine version 2");
    assert_file_read_write!(ws, "tangerine.txt");
    assert_file_content!(ws, "cantaloupe.txt", "cantaloupe version 2");
    assert_file_read_write!(ws, "cantaloupe.txt");

    // Sync back to the latest commit: this should work too.
    let current_commit_id = ws.sync().await.unwrap();
    assert_eq!(current_commit_id, commit_id_2);

    assert_file_content!(ws, "apple.txt", "apple version 2");
    assert_file_read_only!(ws, "apple.txt");
    assert_path_doesnt_exist!(ws, "orange.txt");
    assert_file_content!(ws, "pear.txt", "pear version 1");
    assert_file_read_only!(ws, "pear.txt");
    assert_file_content!(ws, "tangerine.txt", "tangerine version 2");
    assert_file_read_write!(ws, "tangerine.txt");
    assert_file_content!(ws, "cantaloupe.txt", "cantaloupe version 2");
    assert_file_read_write!(ws, "cantaloupe.txt");

    // Changes should still be there.
    assert_staged_changes!(
        ws,
        [
            edit(
                "/cantaloupe.txt",
                "51cc30f874df85b05976c169c23871ddd1a572a4a266b021bba929427e9f1d33",
                "51cc30f874df85b05976c169c23871ddd1a572a4a266b021bba929427e9f1d33",
                20,
                20
            ),
            edit(
                "/tangerine.txt",
                "c753b9ae0bc47041a91eb967401fdeb5c583f64a5ea55a7c1bf381f4634193f5",
                "e39d1f2bdd246ab1f4df9666cfa3a8297cf342f0e8c4cb0827b79c3583848369",
                19,
                19
            ),
        ]
    );
    assert_unstaged_changes!(
        ws,
        [edit(
            "/cantaloupe.txt",
            "51cc30f874df85b05976c169c23871ddd1a572a4a266b021bba929427e9f1d33",
            "d900c4bbbc5836bc7c0cab9e2d639f11324159fc0c37efbda2e7df5b64e40d4a",
            20,
            20
        )]
    );

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_sync_forward_with_conflicting_changes() {
    let (index, ws, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "apple version 1");
    create_file!(ws, "orange.txt", "orange version 1");
    workspace_add_files!(ws, ["."]);
    let commit_id_1 = workspace_commit!(ws, "version 1");

    // Update an existing file.
    workspace_edit_files!(ws, ["apple.txt"]);
    update_file!(ws, "apple.txt", "apple version 2");

    // Create a new one.
    create_file!(ws, "pear.txt", "pear version 1");
    workspace_add_files!(ws, ["apple.txt", "pear.txt"]);

    // And delete an old one.
    workspace_delete_files!(ws, ["orange.txt"]);

    workspace_commit!(ws, "version 2");

    // Make some conflicting change: unstaged first.
    create_file!(ws, "orange.txt", "orange version 2");

    assert_staged_changes!(ws, []);
    assert_unstaged_changes!(
        ws,
        [add(
            "/orange.txt",
            "7b2d4eb0b93dba7b1bc963e6b6f684334b777af220ae6346f8d6355f4da7d80a",
            16
        )]
    );

    // Try to sync back to the previous commit: this should fail as we have
    // unstaged changes about a file that would be restored as part of the sync.
    match ws.sync_to(&commit_id_1).await {
        Err(Error::ConflictingChanges {
            conflicting_changes,
        }) => {
            assert_eq!(
                conflicting_changes,
                [add(
                    "/orange.txt",
                    "7b2d4eb0b93dba7b1bc963e6b6f684334b777af220ae6346f8d6355f4da7d80a",
                    16
                )]
                .into()
            );
        }
        Err(err) => panic!("Unexpected error: {:?}", err),
        Ok(_) => panic!("Expected error, but got success"),
    }

    // Clear the unstaged file.
    delete_file!(ws, "orange.txt");

    // Now make some other conflicting change: staged.
    workspace_edit_files!(ws, ["apple.txt"]);
    update_file!(ws, "apple.txt", "some change");
    workspace_add_files!(ws, ["apple.txt"]);

    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            "0f57832b8352cfebb2d971faca19a2563e3183a53ea935983ae50bcacd8fad76",
            "37e3987705b6b5b750070cf88336f4e93f26a0c35bcb8632b766d4eedac4ab5d",
            15,
            11
        ),]
    );
    assert_unstaged_changes!(ws, []);

    // Try to sync back to the previous commit: this should fail as we have
    // staged changes about a file that would be restored as part of the sync.
    match ws.sync_to(&commit_id_1).await {
        Err(Error::ConflictingChanges {
            conflicting_changes,
        }) => {
            assert_eq!(
                conflicting_changes,
                [edit(
                    "/apple.txt",
                    "0f57832b8352cfebb2d971faca19a2563e3183a53ea935983ae50bcacd8fad76",
                    "37e3987705b6b5b750070cf88336f4e93f26a0c35bcb8632b766d4eedac4ab5d",
                    15,
                    11
                )]
                .into()
            );
        }
        Err(err) => panic!("Unexpected error: {:?}", err),
        Ok(_) => panic!("Expected error, but got success"),
    }

    cleanup_test_workspace_and_index!(ws, index);
}
