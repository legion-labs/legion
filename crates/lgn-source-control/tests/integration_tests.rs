mod common;

#[allow(clippy::wildcard_imports)]
use common::*;

use std::path::Path;

use lgn_content_store2::{Chunker, MemoryProvider, SmallContentProvider};
use lgn_source_control::{
    Error, Index, MapOtherError, Staging, Workspace, WorkspaceConfig, WorkspaceRegistration,
};
use lgn_telemetry_sink::TelemetryGuard;

#[tokio::test]
async fn test_add_and_commit() {
    let _telemetry_guard = TelemetryGuard::default();
    let (index, ws, csp, _paths) = init_test_workspace_and_index!();

    // Add some files.
    create_file!(ws, "apple.txt", "I am an apple");
    create_file!(ws, "orange.txt", "I am an orange");
    create_file!(ws, "vegetables/carrot.txt", "I am a carrot");

    assert_unstaged_changes!(
        ws,
        &csp,
        [
            add("/apple.txt", id(&csp, "I am an apple")),
            add("/orange.txt", id(&csp, "I am an orange")),
            add("/vegetables/carrot.txt", id(&csp, "I am a carrot")),
        ]
    );
    assert_staged_changes!(ws, []);

    let new_added_files = workspace_add_files!(ws, &csp, ["."]);

    assert_eq!(
        new_added_files,
        [
            cp("/apple.txt"),
            cp("/orange.txt"),
            cp("/vegetables/carrot.txt")
        ]
        .into(),
    );

    assert_unstaged_changes!(ws, &csp, []);
    assert_staged_changes!(
        ws,
        [
            add("/apple.txt", id(&csp, "I am an apple")),
            add("/orange.txt", id(&csp, "I am an orange")),
            add("/vegetables/carrot.txt", id(&csp, "I am a carrot")),
        ]
    );

    // Re-adding the same files should be a no-op.
    let new_added_files = workspace_add_files!(ws, &csp, ["."]);

    assert_eq!(new_added_files, [].into());

    assert_staged_changes!(
        ws,
        [
            add("/apple.txt", id(&csp, "I am an apple")),
            add("/orange.txt", id(&csp, "I am an orange")),
            add("/vegetables/carrot.txt", id(&csp, "I am a carrot")),
        ]
    );

    // Editing the same files should be a no-op.
    let new_edited_files = workspace_checkout_files!(ws, &csp, ["."]);

    assert_eq!(new_edited_files, [].into());

    assert_staged_changes!(
        ws,
        [
            add("/apple.txt", id(&csp, "I am an apple")),
            add("/orange.txt", id(&csp, "I am an orange")),
            add("/vegetables/carrot.txt", id(&csp, "I am a carrot")),
        ]
    );

    // Commit the files.
    workspace_commit!(ws, &csp, "Added some fruits");

    assert_staged_changes!(ws, []);
    assert_file_read_only!(ws, "apple.txt");
    assert_file_content!(ws, "apple.txt", "I am an apple");
    assert_file_read_only!(ws, "orange.txt");
    assert_file_content!(ws, "orange.txt", "I am an orange");
    assert_file_read_only!(ws, "vegetables/carrot.txt");
    assert_file_content!(ws, "vegetables/carrot.txt", "I am a carrot");

    // Committing the change should fail, as the commit is empty.
    workspace_commit_error!(ws, &csp, "Edited the carrot", Error::EmptyCommitNotAllowed);

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn lenient_commit() {
    let _telemetry_guard = TelemetryGuard::default();
    let (index, ws, csp, _paths) = init_test_workspace_and_index!();
    create_file!(ws, "apple.txt", "I am an apple");
    create_file!(ws, "orange.txt", "I am an orange");
    create_file!(ws, "vegetables/carrot.txt", "I am a carrot");
    workspace_add_files!(ws, &csp, ["."]);
    workspace_commit!(ws, &csp, "Added some fruits");

    // commit no changes.
    {
        let new_edited_files = ws
            .checkout_files(&csp, [Path::new("vegetables")])
            .await
            .unwrap();
        assert_eq!(new_edited_files, [cp("/vegetables/carrot.txt")].into());

        assert_unstaged_changes!(ws, &csp, []);
        assert_staged_changes!(
            ws,
            [edit(
                "/vegetables/carrot.txt",
                id(&csp, "I am a carrot"),
                id(&csp, "I am a carrot"),
            )]
        );

        assert_file_read_write!(ws, "vegetables/carrot.txt");
        assert_file_content!(ws, "vegetables/carrot.txt", "I am a carrot");

        workspace_commit_lenient!(ws, &csp, "no commit");

        assert_staged_changes!(ws, []);
        assert_unstaged_changes!(ws, &csp, []);

        assert_file_read_only!(ws, "vegetables/carrot.txt");
    }

    // empty commit
    workspace_commit_lenient!(ws, &csp, "empty");

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_edit_and_commit() {
    let _telemetry_guard = TelemetryGuard::default();
    let (index, ws, csp, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am an apple");
    create_file!(ws, "orange.txt", "I am an orange");
    create_file!(ws, "vegetables/carrot.txt", "I am a carrot");
    workspace_add_files!(ws, &csp, ["."]);
    workspace_commit!(ws, &csp, "Added some fruits");

    // Let's now edit one file.
    let new_edited_files = ws
        .checkout_files(&csp, [Path::new("vegetables")])
        .await
        .unwrap();

    assert_eq!(new_edited_files, [cp("/vegetables/carrot.txt")].into());

    assert_unstaged_changes!(ws, &csp, []);
    assert_staged_changes!(
        ws,
        [edit(
            "/vegetables/carrot.txt",
            id(&csp, "I am a carrot"),
            id(&csp, "I am a carrot"),
        )]
    );

    assert_file_read_write!(ws, "vegetables/carrot.txt");
    assert_file_content!(ws, "vegetables/carrot.txt", "I am a carrot");

    update_file!(ws, "vegetables/carrot.txt", "I am a new carrot");

    assert_unstaged_changes!(
        ws,
        &csp,
        [edit(
            "/vegetables/carrot.txt",
            id(&csp, "I am a carrot"),
            id(&csp, "I am a new carrot"),
        )]
    );

    // Committing the change should fail, as we have one edited file whose hash
    // changed but the change was not staged.
    match workspace_commit_error!(ws, &csp, "Edited the carrot") {
        Error::UnchangedFilesMarkedForEdition { paths } => {
            assert_eq!(paths, [cp("/vegetables/carrot.txt")].into());
        }
        e => panic!("Unexpected error: {:?}", e),
    };

    // Add or edit should work the same here: let's first try with add. We'll test with edit later.
    let new_added_files = workspace_add_files!(ws, &csp, ["."]);

    assert_eq!(new_added_files, [cp("/vegetables/carrot.txt")].into());

    assert_staged_changes!(
        ws,
        [edit(
            "/vegetables/carrot.txt",
            id(&csp, "I am a carrot"),
            id(&csp, "I am a new carrot"),
        )]
    );

    // Commit the files.
    workspace_commit!(ws, &csp, "Edited the carrot");

    assert_staged_changes!(ws, []);
    assert_file_read_only!(ws, "vegetables/carrot.txt");
    assert_file_content!(ws, "vegetables/carrot.txt", "I am a new carrot");
    assert_file_content!(ws, "apple.txt", "I am an apple");

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_delete_and_commit() {
    let _telemetry_guard = TelemetryGuard::default();
    let (index, ws, csp, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am an apple");
    create_file!(ws, "orange.txt", "I am an orange");
    create_file!(ws, "vegetables/carrot.txt", "I am a new carrot");
    workspace_add_files!(ws, &csp, ["."]);
    workspace_commit!(ws, &csp, "Added some fruits");

    // Let's delete a file.
    let new_deleted_files = workspace_delete_files!(ws, &csp, ["vegetables/carrot.txt"]);

    assert_eq!(new_deleted_files, [cp("/vegetables/carrot.txt")].into());

    assert_staged_changes!(
        ws,
        [delete(
            "/vegetables/carrot.txt",
            id(&csp, "I am a new carrot"),
        )]
    );

    assert_path_doesnt_exist!(ws, "vegetables/carrot.txt");
    assert_file_content!(ws, "apple.txt", "I am an apple");

    // Commit the files.
    workspace_commit!(ws, &csp, "Removed the carrot");

    assert_staged_changes!(ws, []);
    assert_path_doesnt_exist!(ws, "vegetables/carrot.txt");

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_add_empty_directory() {
    let _telemetry_guard = TelemetryGuard::default();
    let (index, ws, csp, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am an apple");
    workspace_add_files!(ws, &csp, ["."]);
    workspace_commit!(ws, &csp, "Added some fruits");

    create_dir!(ws, "vegetables");

    // Adding an empty directory but existing should yield no error and add no files.
    let new_added_files = ws.add_files(&csp, [Path::new("vegetables")]).await.unwrap();

    assert_eq!(new_added_files, [].into(),);

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_add_non_existing_path() {
    let _telemetry_guard = TelemetryGuard::default();
    let (index, ws, csp, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am an apple");
    workspace_add_files!(ws, &csp, ["."]);
    workspace_commit!(ws, &csp, "Added some fruits");

    // Adding an non-existing path should fail.
    match ws.add_files(&csp, [Path::new("non/existing/path")]).await {
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
    let _telemetry_guard = TelemetryGuard::default();
    let (index, ws, csp, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am an apple");
    create_file!(ws, "orange.txt", "I am an orange");
    workspace_add_files!(ws, &csp, ["."]);
    workspace_commit!(ws, &csp, "Added some fruits");

    create_file!(ws, "banana.txt", "I am a banana");

    let new_added_files = workspace_add_files!(ws, &csp, ["."]);

    assert_eq!(new_added_files, [cp("/banana.txt")].into());

    let new_deleted_files = workspace_delete_files!(ws, &csp, ["banana.txt"]);

    assert_eq!(new_deleted_files, [cp("/banana.txt")].into());
    assert_path_doesnt_exist!(ws, "banana.txt");

    // The file was not really in the tree: it was just staged for addition. So its removal is actually a no-op.
    assert_unstaged_changes!(ws, &csp, []);
    assert_staged_changes!(ws, []);

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_edit_and_commit_with_extra_unstaged_changes_then_revert() {
    let _telemetry_guard = TelemetryGuard::default();
    let (index, ws, csp, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am an apple");
    create_file!(ws, "orange.txt", "I am an orange");
    workspace_add_files!(ws, &csp, ["."]);
    workspace_commit!(ws, &csp, "Added some fruits");

    let new_edited_files = workspace_checkout_files!(ws, &csp, ["apple.txt"]);

    assert_eq!(new_edited_files, [cp("/apple.txt")].into());

    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            id(&csp, "I am an apple"),
            id(&csp, "I am an apple"),
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
            id(&csp, "I am an apple"),
            id(&csp, "I am an apple"),
        )]
    );

    let new_added_files = workspace_add_files!(ws, &csp, ["apple.txt"]);

    assert_eq!(new_added_files, [cp("/apple.txt")].into());

    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            id(&csp, "I am an apple"),
            id(&csp, "I am a new apple"),
        )]
    );

    // Let's make another change but do not stage it.
    update_file!(ws, "apple.txt", "I am an even newer apple");

    // Commit the files.
    workspace_commit!(ws, &csp, "Update the apple");

    // File should not be read only nor unlocked as it has unstaged changes.
    assert_file_read_write!(ws, "apple.txt");
    assert_file_content!(ws, "apple.txt", "I am an even newer apple");

    // We should still have some unstaged changes as we did not stage them yet.
    // Also the file should still be checked out for edition and locked.
    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            id(&csp, "I am a new apple"),
            id(&csp, "I am a new apple"),
        )]
    );
    assert_unstaged_changes!(
        ws,
        &csp,
        [edit(
            "/apple.txt",
            id(&csp, "I am a new apple"),
            id(&csp, "I am an even newer apple"),
        )]
    );

    // Reverting the file with unstaged changes should work.
    let new_reverted_files =
        workspace_revert_files!(ws, &csp, ["apple.txt"], Staging::StagedAndUnstaged);

    assert_eq!(new_reverted_files, [cp("/apple.txt")].into());

    assert_staged_changes!(ws, []);
    assert_unstaged_changes!(ws, &csp, []);

    assert_file_read_only!(ws, "apple.txt");
    assert_file_content!(ws, "apple.txt", "I am a new apple");

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_revert_after_add_and_edit() {
    let _telemetry_guard = TelemetryGuard::default();
    let (index, ws, csp, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am a new apple");
    create_file!(ws, "orange.txt", "I am an orange");
    workspace_add_files!(ws, &csp, ["."]);
    workspace_commit!(ws, &csp, "Added some fruits");

    // Let's make a change but stage it this time.
    let new_edited_files = workspace_checkout_files!(ws, &csp, ["apple.txt"]);

    assert_eq!(new_edited_files, [cp("/apple.txt"),].into());

    update_file!(ws, "apple.txt", "I am an even newer apple");
    create_file!(ws, "strawberry.txt", "I am a strawberry");

    let new_added_files = workspace_add_files!(ws, &csp, ["."]);

    assert_eq!(
        new_added_files,
        [cp("/apple.txt"), cp("/strawberry.txt")].into()
    );

    assert_staged_changes!(
        ws,
        [
            edit(
                "/apple.txt",
                id(&csp, "I am a new apple"),
                id(&csp, "I am an even newer apple"),
            ),
            add("/strawberry.txt", id(&csp, "I am a strawberry")),
        ]
    );
    assert_unstaged_changes!(ws, &csp, []);

    // Reverting the file with staged changes should work too.
    let new_reverted_files = workspace_revert_files!(ws, &csp, ["."], Staging::StagedAndUnstaged);

    assert_eq!(
        new_reverted_files,
        [cp("/apple.txt"), cp("/strawberry.txt")].into()
    );

    // Untracked files that are reverted for add are not deleted.
    assert_staged_changes!(ws, []);
    assert_unstaged_changes!(
        ws,
        &csp,
        [add("/strawberry.txt", id(&csp, "I am a strawberry"))]
    );

    assert_file_read_only!(ws, "apple.txt");
    assert_file_read_write!(ws, "strawberry.txt");
    assert_file_content!(ws, "apple.txt", "I am a new apple");
    assert_file_content!(ws, "strawberry.txt", "I am a strawberry");

    // Let's delete the file: it should not appear unstaged anymore.
    delete_file!(ws, "strawberry.txt");

    assert_staged_changes!(ws, []);
    assert_unstaged_changes!(ws, &csp, []);

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_revert_staged_only_with_unstaged_changes() {
    let _telemetry_guard = TelemetryGuard::default();
    let (index, ws, csp, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am a new apple");
    create_file!(ws, "orange.txt", "I am an orange");
    workspace_add_files!(ws, &csp, ["."]);
    workspace_commit!(ws, &csp, "Added some fruits");

    // Let's mark a file for edition, change it but do not stage it. Then let's revert it in staging.
    let new_edited_files = workspace_checkout_files!(ws, &csp, ["apple.txt"]);

    assert_eq!(new_edited_files, [cp("/apple.txt")].into());
    update_file!(ws, "apple.txt", "I am an even newer apple");

    assert_unstaged_changes!(
        ws,
        &csp,
        [edit(
            "/apple.txt",
            id(&csp, "I am a new apple"),
            id(&csp, "I am an even newer apple"),
        )]
    );
    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            id(&csp, "I am a new apple"),
            id(&csp, "I am a new apple"),
        )]
    );

    // The file has unstaged changes, so reverting in staged-only mode should
    // not affect it.
    let new_reverted_files = workspace_revert_files!(ws, &csp, ["apple.txt"], Staging::StagedOnly);

    assert_eq!(new_reverted_files, [].into());

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_revert_staged_only_with_staged_and_unstaged_changes() {
    let _telemetry_guard = TelemetryGuard::default();
    let (index, ws, csp, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am a new apple");
    create_file!(ws, "orange.txt", "I am an orange");
    workspace_add_files!(ws, &csp, ["."]);
    workspace_commit!(ws, &csp, "Added some fruits");

    let new_edited_files = workspace_checkout_files!(ws, &csp, ["apple.txt"]);

    assert_eq!(new_edited_files, [cp("/apple.txt")].into());
    update_file!(ws, "apple.txt", "I am an even newer apple");

    let new_added_files = workspace_add_files!(ws, &csp, ["apple.txt"]);

    assert_eq!(new_added_files, [cp("/apple.txt"),].into());

    assert_unstaged_changes!(ws, &csp, []);
    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            id(&csp, "I am a new apple"),
            id(&csp, "I am an even newer apple"),
        )]
    );

    update_file!(ws, "apple.txt", "Unstaged modification");

    let new_reverted_files = workspace_revert_files!(ws, &csp, ["apple.txt"], Staging::StagedOnly);

    assert_eq!(new_reverted_files, [cp("/apple.txt")].into());

    assert_file_read_write!(ws, "apple.txt");
    assert_file_content!(ws, "apple.txt", "Unstaged modification");

    assert_unstaged_changes!(
        ws,
        &csp,
        [edit(
            "/apple.txt",
            id(&csp, "I am a new apple"),
            id(&csp, "Unstaged modification"),
        )]
    );
    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            id(&csp, "I am a new apple"),
            id(&csp, "I am a new apple"),
        )]
    );

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_revert_unstaged_only_with_unstaged_changes() {
    let _telemetry_guard = TelemetryGuard::default();
    let (index, ws, csp, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "I am a new apple");
    create_file!(ws, "orange.txt", "I am an orange");
    workspace_add_files!(ws, &csp, ["."]);
    workspace_commit!(ws, &csp, "Added some fruits");

    let new_edited_files = workspace_checkout_files!(ws, &csp, ["apple.txt"]);

    assert_eq!(new_edited_files, [cp("/apple.txt")].into());
    update_file!(ws, "apple.txt", "Unstaged modification");

    // This time let's only revert unstaged changes.
    let new_reverted_files =
        workspace_revert_files!(ws, &csp, ["apple.txt"], Staging::UnstagedOnly);

    assert_eq!(new_reverted_files, [cp("/apple.txt")].into());

    assert_file_read_write!(ws, "apple.txt");
    assert_file_content!(ws, "apple.txt", "I am a new apple");

    assert_unstaged_changes!(ws, &csp, []);
    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            id(&csp, "I am a new apple"),
            id(&csp, "I am a new apple"),
        )]
    );

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_sync_forward_and_backward() {
    let _telemetry_guard = TelemetryGuard::default();
    let (index, ws, csp, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "apple version 1");
    create_file!(ws, "orange.txt", "orange version 1");
    workspace_add_files!(ws, &csp, ["."]);
    let commit_1 = workspace_commit!(ws, &csp, "version 1");

    // Update an existing file.
    workspace_checkout_files!(ws, &csp, ["apple.txt"]);
    update_file!(ws, "apple.txt", "apple version 2");

    // Create a new one.
    create_file!(ws, "pear.txt", "pear version 1");
    workspace_add_files!(ws, &csp, ["apple.txt", "pear.txt"]);

    // And delete an old one.
    workspace_delete_files!(ws, &csp, ["orange.txt"]);

    let commit_2 = workspace_commit!(ws, &csp, "version 2");

    assert_file_content!(ws, "apple.txt", "apple version 2");
    assert_file_read_only!(ws, "apple.txt");
    assert_path_doesnt_exist!(ws, "orange.txt");
    assert_file_content!(ws, "pear.txt", "pear version 1");
    assert_file_read_only!(ws, "pear.txt");

    // Try to sync back to the previous commit.
    let changes = ws.sync_to(&csp, commit_1.id).await.unwrap();

    assert_eq!(
        changes,
        [
            add("/orange.txt", id(&csp, "orange version 1"),),
            edit(
                "/apple.txt",
                id(&csp, "apple version 2"),
                id(&csp, "apple version 1"),
            ),
            delete("/pear.txt", id(&csp, "pear version 1"),),
        ]
        .into()
    );

    assert_path_doesnt_exist!(ws, "pear.txt");
    assert_file_content!(ws, "apple.txt", "apple version 1");
    assert_file_read_only!(ws, "apple.txt");
    assert_file_content!(ws, "orange.txt", "orange version 1");
    assert_file_read_only!(ws, "orange.txt");

    // Sync back to the latest commit.
    let (branch, changes) = ws.sync(&csp).await.unwrap();

    assert_eq!(branch.head, commit_2.id);
    assert_eq!(
        changes,
        [
            delete("/orange.txt", id(&csp, "orange version 1"),),
            edit(
                "/apple.txt",
                id(&csp, "apple version 1"),
                id(&csp, "apple version 2"),
            ),
            add("/pear.txt", id(&csp, "pear version 1"),),
        ]
        .into()
    );

    assert_file_content!(ws, "apple.txt", "apple version 2");
    assert_file_read_only!(ws, "apple.txt");
    assert_path_doesnt_exist!(ws, "orange.txt");
    assert_file_content!(ws, "pear.txt", "pear version 1");
    assert_file_read_only!(ws, "pear.txt");

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_sync_forward_with_non_conflicting_changes() {
    let _telemetry_guard = TelemetryGuard::default();
    let (index, ws, csp, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "tangerine.txt", "tangerine version 1");
    create_file!(ws, "cantaloupe.txt", "cantaloupe version 1");
    create_file!(ws, "apple.txt", "apple version 1");
    create_file!(ws, "orange.txt", "orange version 1");
    workspace_add_files!(ws, &csp, ["."]);
    let commit_1 = workspace_commit!(ws, &csp, "version 1");

    // Update an existing file.
    workspace_checkout_files!(ws, &csp, ["apple.txt"]);
    update_file!(ws, "apple.txt", "apple version 2");

    // Create a new one.
    create_file!(ws, "pear.txt", "pear version 1");
    workspace_add_files!(ws, &csp, ["apple.txt", "pear.txt"]);

    // And delete an old one.
    workspace_delete_files!(ws, &csp, ["orange.txt"]);

    let commit_2 = workspace_commit!(ws, &csp, "version 2");

    // Make some staged and unstaged changes.
    workspace_checkout_files!(ws, &csp, ["tangerine.txt", "cantaloupe.txt"]);
    update_file!(ws, "tangerine.txt", "tangerine version 2");
    update_file!(ws, "cantaloupe.txt", "cantaloupe version 2");
    workspace_add_files!(ws, &csp, ["tangerine.txt"]);

    assert_staged_changes!(
        ws,
        [
            edit(
                "/cantaloupe.txt",
                id(&csp, "cantaloupe version 1"),
                id(&csp, "cantaloupe version 1"),
            ),
            edit(
                "/tangerine.txt",
                id(&csp, "tangerine version 1"),
                id(&csp, "tangerine version 2"),
            ),
        ]
    );
    assert_unstaged_changes!(
        ws,
        &csp,
        [edit(
            "/cantaloupe.txt",
            id(&csp, "cantaloupe version 1"),
            id(&csp, "cantaloupe version 2"),
        )]
    );

    // Try to sync back to the previous commit: this should work even though we
    // have local changes as those do not conflict at all.
    let changes = ws.sync_to(&csp, commit_1.id).await.unwrap();

    assert_eq!(
        changes,
        [
            add("/orange.txt", id(&csp, "orange version 1"),),
            edit(
                "/apple.txt",
                id(&csp, "apple version 2"),
                id(&csp, "apple version 1"),
            ),
            delete("/pear.txt", id(&csp, "pear version 1"),),
        ]
        .into()
    );

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
    let (branch, changes) = ws.sync(&csp).await.unwrap();

    assert_eq!(branch.head, commit_2.id);
    assert_eq!(
        changes,
        [
            delete("/orange.txt", id(&csp, "orange version 1"),),
            edit(
                "/apple.txt",
                id(&csp, "apple version 1"),
                id(&csp, "apple version 2"),
            ),
            add("/pear.txt", id(&csp, "pear version 1"),),
        ]
        .into()
    );

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
                id(&csp, "cantaloupe version 1"),
                id(&csp, "cantaloupe version 1"),
            ),
            edit(
                "/tangerine.txt",
                id(&csp, "tangerine version 1"),
                id(&csp, "tangerine version 2"),
            ),
        ]
    );
    assert_unstaged_changes!(
        ws,
        &csp,
        [edit(
            "/cantaloupe.txt",
            id(&csp, "cantaloupe version 1"),
            id(&csp, "cantaloupe version 2"),
        )]
    );

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_sync_forward_with_conflicting_changes() {
    let _telemetry_guard = TelemetryGuard::default();
    let (index, ws, csp, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "apple version 1");
    create_file!(ws, "orange.txt", "orange version 1");
    workspace_add_files!(ws, &csp, ["."]);
    let commit_id_1 = workspace_commit!(ws, &csp, "version 1").id;

    // Update an existing file.
    workspace_checkout_files!(ws, &csp, ["apple.txt"]);
    update_file!(ws, "apple.txt", "apple version 2");

    // Create a new one.
    create_file!(ws, "pear.txt", "pear version 1");
    workspace_add_files!(ws, &csp, ["apple.txt", "pear.txt"]);

    // And delete an old one.
    workspace_delete_files!(ws, &csp, ["orange.txt"]);

    workspace_commit!(ws, &csp, "version 2");

    // Make some conflicting change: unstaged first.
    create_file!(ws, "orange.txt", "orange version 2");

    assert_staged_changes!(ws, []);
    assert_unstaged_changes!(
        ws,
        &csp,
        [add("/orange.txt", id(&csp, "orange version 2"),)]
    );

    // Try to sync back to the previous commit: this should fail as we have
    // unstaged changes about a file that would be restored as part of the sync.
    match ws.sync_to(&csp, commit_id_1).await {
        Err(Error::ConflictingChanges {
            conflicting_changes,
        }) => {
            assert_eq!(
                conflicting_changes,
                [add("/orange.txt", id(&csp, "orange version 2"),)].into()
            );
        }
        Err(err) => panic!("Unexpected error: {:?}", err),
        Ok(_) => panic!("Expected error, but got success"),
    }

    // Clear the unstaged file.
    delete_file!(ws, "orange.txt");

    // Now make some other conflicting change: staged.
    workspace_checkout_files!(ws, &csp, ["apple.txt"]);
    update_file!(ws, "apple.txt", "some change");
    workspace_add_files!(ws, &csp, ["apple.txt"]);

    assert_staged_changes!(
        ws,
        [edit(
            "/apple.txt",
            id(&csp, "apple version 2"),
            id(&csp, "some change"),
        ),]
    );
    assert_unstaged_changes!(ws, &csp, []);

    // Try to sync back to the previous commit: this should fail as we have
    // staged changes about a file that would be restored as part of the sync.
    match ws.sync_to(&csp, commit_id_1).await {
        Err(Error::ConflictingChanges {
            conflicting_changes,
        }) => {
            assert_eq!(
                conflicting_changes,
                [edit(
                    "/apple.txt",
                    id(&csp, "apple version 2"),
                    id(&csp, "some change"),
                )]
                .into()
            );
        }
        Err(err) => panic!("Unexpected error: {:?}", err),
        Ok(_) => panic!("Expected error, but got success"),
    }

    cleanup_test_workspace_and_index!(ws, index);
}

#[tokio::test]
async fn test_create_branch_switch_detach_attach() {
    let _telemetry_guard = TelemetryGuard::default();
    let (index, ws, csp, _paths) = init_test_workspace_and_index!();

    create_file!(ws, "apple.txt", "apple version 1");
    create_file!(ws, "orange.txt", "orange version 1");
    workspace_add_files!(ws, &csp, ["."]);
    workspace_commit!(ws, &csp, "version 1");

    let main = workspace_get_current_branch!(ws);
    let mut branch1 = workspace_create_branch!(ws, "branch1");

    assert_eq!(branch1, main.branch_out("branch1".to_string()));

    // Make sure we really did switch off to the new branch.
    assert_workspace_current_branch!(ws, branch1);

    // Make and edit and commit.
    workspace_checkout_files!(ws, &csp, ["apple.txt"]);
    update_file!(ws, "apple.txt", "apple version 2");
    workspace_add_files!(ws, &csp, ["apple.txt"]);

    branch1.head = workspace_commit!(ws, &csp, "version 2").id;

    // Go back to the main branch.
    workspace_switch_branch!(ws, &csp, "main");

    // Make sure we really did switch back to the main branch.
    assert_workspace_current_branch!(ws, main);

    assert_file_content!(ws, "apple.txt", "apple version 1");
    assert_file_read_only!(ws, "apple.txt");

    // Go back to the branch1 branch.
    workspace_switch_branch!(ws, &csp, "branch1");

    // Make sure we really did switch back to the main branch.
    assert_workspace_current_branch!(ws, branch1);

    assert_file_content!(ws, "apple.txt", "apple version 2");
    assert_file_read_only!(ws, "apple.txt");

    cleanup_test_workspace_and_index!(ws, index);
}
