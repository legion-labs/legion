mod common;

#[allow(clippy::wildcard_imports)]
use common::*;

use std::path::Path;

use lgn_source_control::{
    Error, Index, MapOtherError, Staging, Workspace, WorkspaceConfig, WorkspaceRegistration,
};

#[tokio::test]
async fn test_add_and_commit() {
    let (_index, ws, _paths) = init_test_workspace_and_index!();

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
    let (_index, ws, _paths) = init_test_workspace_and_index!();

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
    let (_index, ws, _paths) = init_test_workspace_and_index!();

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
    let (_index, ws, _paths) = init_test_workspace_and_index!();

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
    let (_index, ws, _paths) = init_test_workspace_and_index!();

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
    let (_index, ws, _paths) = init_test_workspace_and_index!();

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
    let (_index, ws, _paths) = init_test_workspace_and_index!();

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
    assert_staged_changes!(ws, []);
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
    let (_index, ws, _paths) = init_test_workspace_and_index!();

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
    let (_index, ws, _paths) = init_test_workspace_and_index!();

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
    let (_index, ws, _paths) = init_test_workspace_and_index!();

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
    let (_index, ws, _paths) = init_test_workspace_and_index!();

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
