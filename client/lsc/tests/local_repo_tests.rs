use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use lgn_telemetry_sink::TelemetryGuard;
use lgn_test_utils::{create_test_dir, syscall};
use lgn_tracing::dispatch::{flush_log_buffer, flush_metrics_buffer, flush_thread_buffer};
use lgn_tracing::{trace_function, trace_scope};

fn write_file(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
    let path = path.as_ref();
    let contents = contents.as_ref();

    fs::create_dir_all(path.parent().unwrap())?;

    fs::File::create(path)?
        .write_all(contents)
        .map_err(Into::into)
}

fn write_lorem_ipsum(p: &Path) {
    let contents = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. In iaculis odio ac nulla porta, eget dictum nulla euismod. Vivamus congue eros vitae velit feugiat lacinia. Curabitur mi lectus, semper in posuere nec, eleifend eu magna. Morbi egestas magna eget ligula aliquet, vitae mattis urna pellentesque. Maecenas sem risus, scelerisque id semper ut, ornare id diam. Integer ut urna varius, lobortis sapien id, ullamcorper mi. Donec pulvinar ante ligula, in interdum turpis tempor a. Maecenas malesuada turpis orci, vitae efficitur tortor laoreet sit amet.

Nulla eu scelerisque odio. Suspendisse ultrices convallis hendrerit. Duis lacinia lacus ut urna pellentesque, euismod auctor risus volutpat. Sed et congue dolor, et bibendum dolor. Nam sit amet ante id eros aliquet luctus. Donec pulvinar mauris turpis, a ullamcorper mi fermentum ac. Morbi a volutpat turpis. Nulla facilisi. Sed rutrum placerat nisl vitae condimentum. Nunc et lacus ut lacus aliquet tempor et volutpat mi. Maecenas pretium ultricies mi id vestibulum. Sed turpis justo, semper eu nisl ac, hendrerit mattis turpis. Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Praesent condimentum pellentesque vestibulum. Fusce at hendrerit lorem.\n";

    write_file(p, contents.as_bytes()).expect("write failed");
}

#[trace_function]
fn append_text_to_file(p: &Path, contents: &str) {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(p)
        .unwrap();
    f.write_all(contents.as_bytes()).unwrap();
}

static LSC_CLI_EXE_VAR: &str = env!("CARGO_BIN_EXE_lsc");
#[trace_function]
fn lsc_cli_sys(wd: &Path, args: &[&str]) {
    syscall(LSC_CLI_EXE_VAR, wd, args, true);
}

#[trace_function]
fn lsc_cli_sys_fail(wd: &Path, args: &[&str]) {
    syscall(LSC_CLI_EXE_VAR, wd, args, false);
}

#[trace_function]
fn init_test_repo(test_dir: &Path, name: &str) -> String {
    if let Ok(test_host_uri) = std::env::var("lgn_source_control_TEST_HOST") {
        let repo_uri = format!("{}/{}", test_host_uri, name);

        let _status = Command::new(LSC_CLI_EXE_VAR)
            .current_dir(test_dir)
            .args(["destroy-repository", &repo_uri])
            .status()
            .expect("failed to execute command");

        match std::env::var("lgn_source_control_TEST_BLOB_STORAGE") {
            Ok(blob_storage_uri) => lsc_cli_sys(
                test_dir,
                &["init-remote-repository", &repo_uri, &blob_storage_uri],
            ),
            Err(_) => lsc_cli_sys(test_dir, &["create-repository", &repo_uri]),
        }
        repo_uri
    } else {
        let repo_dir = test_dir.join("repo");
        lsc_cli_sys(test_dir, &["create-repository", repo_dir.to_str().unwrap()]);
        String::from(repo_dir.to_str().unwrap())
    }
}

#[trace_function]
fn init_test_dir(test_name: &str) -> PathBuf {
    let parent = Path::new(LSC_CLI_EXE_VAR)
        .parent()
        .unwrap()
        .join("lsc-test-scratch");
    create_test_dir(&parent, test_name)
}

#[test]
fn local_repo_suite() {
    let telemetry_guard = TelemetryGuard::new();
    std::mem::forget(telemetry_guard);
    trace_scope!("lsc::local_repo_suite");
    let test_dir = init_test_dir("local_repo_suite");
    let work1 = test_dir.join("work");
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let repo_uri = init_test_repo(&test_dir, "local_repo_suite");

    lsc_cli_sys(
        &test_dir,
        &["init-workspace", work1.to_str().unwrap(), &repo_uri],
    );

    std::fs::create_dir_all(work1.join("dir0/deep")).expect("dir0/deep creation failed");
    write_lorem_ipsum(&work1.join("dir0/file0.txt"));
    write_lorem_ipsum(&work1.join("dir0/file1.txt"));
    write_lorem_ipsum(&work1.join("dir0/deep/file2.txt"));
    std::fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/lambda.jpg"),
        work1.join("bin.jpg"),
    )
    .expect("error copying lambda.jpg");

    lsc_cli_sys(
        &work1,
        &["add", work1.join("dir0/file0.txt").to_str().unwrap()],
    );

    lsc_cli_sys(&work1, &["add", "dir0/file1.txt"]);
    lsc_cli_sys(&work1, &["add", "dir0/deep/file2.txt"]);
    lsc_cli_sys(&work1, &["add", "bin.jpg"]);
    lsc_cli_sys(&work1, &["local-changes"]);
    lsc_cli_sys(&work1, &["commit", r#"-m"my commit message""#]);

    write_lorem_ipsum(&work1.join("dir0/file3.txt"));
    lsc_cli_sys(&work1, &["add", "dir0/file3.txt"]);
    lsc_cli_sys(&work1, &["local-changes"]);
    lsc_cli_sys(&work1, &["commit", r#"-m"my second commit message""#]);

    let work2 = test_dir.join("work2");
    lsc_cli_sys(
        &test_dir,
        &["init-workspace", work2.to_str().unwrap(), &repo_uri],
    );
    assert!(fs::metadata(work2.join("dir0/file3.txt")).is_ok());
    assert!(fs::metadata(work2.join("dir0/file1.txt")).is_ok());

    //still under first workspace
    lsc_cli_sys(&work1, &["log"]);
    lsc_cli_sys(&work1, &["edit", "dir0/file0.txt"]);
    lsc_cli_sys(&work1, &["local-changes"]);
    append_text_to_file(&work1.join("dir0/file0.txt"), "\nnew line in file0");
    lsc_cli_sys(&work1, &["diff", "--notool", "dir0/file0.txt"]);
    lsc_cli_sys(&work1, &["diff", "--notool", "dir0/file0.txt", "latest"]);
    lsc_cli_sys(&work1, &["commit", r#"-m"edit file0""#]);

    lsc_cli_sys(&work1, &["edit", "dir0/file1.txt"]);
    append_text_to_file(&work1.join("dir0/file1.txt"), "\nnew line in file1");
    lsc_cli_sys(&work1, &["commit", r#"-m"edit file1""#]);

    lsc_cli_sys(&work2, &["log"]);

    //should not be allowed to commit before sync
    lsc_cli_sys(&work2, &["edit", "dir0/file0.txt"]);
    lsc_cli_sys_fail(&work2, &["commit", r#"-m"bad commit""#]);
    lsc_cli_sys(&work2, &["revert", "dir0/file0.txt"]);
    lsc_cli_sys(&work2, &["sync"]);

    lsc_cli_sys(&work2, &["delete", "dir0/file0.txt"]);
    lsc_cli_sys(&work2, &["local-changes"]);
    lsc_cli_sys(&work2, &["log"]);
    lsc_cli_sys(&work2, &["commit", r#"-m"delete file0""#]);

    // Switching back to wd1
    lsc_cli_sys(&work1, &["log"]);
    lsc_cli_sys(&work1, &["sync"]);

    //test revert add
    write_lorem_ipsum(&work1.join("dir0/added_and_reverted.txt"));
    lsc_cli_sys(&work1, &["add", "dir0/added_and_reverted.txt"]);
    lsc_cli_sys(&work1, &["revert", "dir0/added_and_reverted.txt"]);

    //test revert delete
    lsc_cli_sys(&work1, &["delete", "dir0/file1.txt"]);
    lsc_cli_sys(&work1, &["revert", "dir0/file1.txt"]);

    //sync backwards
    let workspace_spec = lgn_source_control::read_workspace_spec(&work1).unwrap();
    let connection = tokio_runtime
        .block_on(lgn_source_control::connect_to_server(&workspace_spec))
        .unwrap();
    let query = connection.query();
    let main_branch = tokio_runtime.block_on(query.read_branch("main")).unwrap();
    let log_vec = tokio_runtime
        .block_on(lgn_source_control::find_branch_commits(
            &connection,
            &main_branch,
        ))
        .unwrap();
    let init_commit = log_vec.last().unwrap();
    lsc_cli_sys(&work1, &["sync", &init_commit.id]);

    //sync forwards
    lsc_cli_sys(&work1, &["sync"]);
    flush_log_buffer();
    flush_metrics_buffer();
    flush_thread_buffer();
}

#[test]
fn local_single_branch_merge_flow() {
    let telemetry_guard = TelemetryGuard::new();
    std::mem::forget(telemetry_guard);
    trace_scope!("lsc::local_single_branch_merge_flow");
    let test_dir = init_test_dir("local_single_branch_merge_flow");
    let work1 = test_dir.join("work1");
    let work2 = test_dir.join("work2");
    let repo_uri = init_test_repo(&test_dir, "local_single_branch_merge_flow");

    lsc_cli_sys(
        &test_dir,
        &["init-workspace", work1.to_str().unwrap(), &repo_uri],
    );

    write_file(&work1.join("file1.txt"), b"line1\n").unwrap();
    lsc_cli_sys(&work1, &["add", "file1.txt"]);
    lsc_cli_sys(&work1, &["commit", r#"-m"add file1""#]);

    lsc_cli_sys(
        &test_dir,
        &["init-workspace", work2.to_str().unwrap(), &repo_uri],
    );

    lsc_cli_sys(&work2, &["edit", "file1.txt"]);
    append_text_to_file(&work2.join("file1.txt"), "line2\n");
    lsc_cli_sys(&work2, &["commit", r#"-m"add line2 to file1""#]);

    lsc_cli_sys(&work1, &["edit", "file1.txt"]);
    append_text_to_file(&work1.join("file1.txt"), "line2\nline3\n");
    lsc_cli_sys_fail(&work1, &["commit", r#"-m"should fail - not at head""#]);
    lsc_cli_sys(&work1, &["sync"]);
    lsc_cli_sys(&work1, &["resolves-pending"]);
    lsc_cli_sys(&work1, &["resolve", "--notool", "file1.txt"]);
    lsc_cli_sys(&work1, &["resolves-pending"]);
    lsc_cli_sys(&work1, &["commit", r#"-m"merged""#]);

    lsc_cli_sys(&work2, &["sync"]);
    lsc_cli_sys(&work2, &["edit", "file1.txt"]);
    append_text_to_file(&work2.join("file1.txt"), "line4\n");
    lsc_cli_sys(&work2, &["commit", r#"-m"add line4 to file1""#]);

    lsc_cli_sys(&work1, &["edit", "file1.txt"]);
    lsc_cli_sys(&work1, &["sync"]);
    lsc_cli_sys(&work1, &["resolves-pending"]);
    lsc_cli_sys(&work1, &["revert", "file1.txt"]);
    lsc_cli_sys(&work1, &["resolves-pending"]);
    flush_log_buffer();
    flush_metrics_buffer();
    flush_thread_buffer();
}

#[test]
fn test_print_config() {
    let telemetry_guard = TelemetryGuard::new();
    std::mem::forget(telemetry_guard);
    trace_scope!("lsc::test_print_config");
    let config_file_path = lgn_source_control::Config::config_file_path().unwrap();
    if config_file_path.exists() {
        lsc_cli_sys(std::env::current_dir().unwrap().as_path(), &["config"]);
    } else {
        println!("no config file, skipping test");
    }
    flush_log_buffer();
    flush_metrics_buffer();
    flush_thread_buffer();
}

#[test]
fn test_branch() {
    let telemetry_guard = TelemetryGuard::new();
    std::mem::forget(telemetry_guard);
    trace_scope!("lsc::test_branch");
    let test_dir = init_test_dir("test_branch");
    let config_file_path = lgn_source_control::Config::config_file_path().unwrap();
    if config_file_path.exists() {
        lsc_cli_sys(&test_dir, &["config"]);
    } else {
        println!("no config file, skipping test");
    }

    let work1 = test_dir.join("work1");
    let repo_uri = init_test_repo(&test_dir, "test_branch");

    lsc_cli_sys(
        &test_dir,
        &["init-workspace", work1.to_str().unwrap(), &repo_uri],
    );

    write_file(&work1.join("file1.txt"), b"line1\n").unwrap();
    lsc_cli_sys(&work1, &["add", "file1.txt"]);

    write_file(&work1.join("file2.txt"), b"line1\n").unwrap();
    lsc_cli_sys(&work1, &["add", "file2.txt"]);

    lsc_cli_sys(&work1, &["commit", r#"-m"add file1""#]);
    lsc_cli_sys(&work1, &["create-branch", "task"]);
    lsc_cli_sys(&work1, &["edit", "file1.txt"]);
    append_text_to_file(&work1.join("file1.txt"), "from task branch\n");

    lsc_cli_sys(&work1, &["delete", "file2.txt"]);

    write_file(&work1.join("file3.txt"), b"line1\n").unwrap();
    lsc_cli_sys(&work1, &["add", "file3.txt"]);

    std::fs::create_dir_all(work1.join("dir0/deep")).expect("dir0 creation failed");
    write_file(&work1.join("dir0/deep/inner_task.txt"), b"line1\n").unwrap();
    lsc_cli_sys(&work1, &["add", "dir0/deep/inner_task.txt"]);

    lsc_cli_sys(&work1, &["commit", r#"-m"task complete""#]);
    lsc_cli_sys(&work1, &["log"]);
    lsc_cli_sys(&work1, &["switch-branch", "main"]);
    lsc_cli_sys(&work1, &["log"]);

    lsc_cli_sys(&work1, &["switch-branch", "task"]);
    lsc_cli_sys(&work1, &["log"]);
    lsc_cli_sys(&work1, &["list-branches"]);

    lsc_cli_sys(&work1, &["switch-branch", "main"]);
    lsc_cli_sys(&work1, &["merge-branch", "task"]); //fast-forward merge

    lsc_cli_sys(&work1, &["edit", "file1.txt"]);
    append_text_to_file(&work1.join("file1.txt"), "from main branch\n");
    lsc_cli_sys(&work1, &["commit", r#"-m"work on main""#]);

    lsc_cli_sys(&work1, &["switch-branch", "task"]);
    lsc_cli_sys(&work1, &["edit", "file3.txt"]);
    append_text_to_file(&work1.join("file3.txt"), "from task branch\n");
    lsc_cli_sys(&work1, &["commit", r#"-m"work on task""#]);
    lsc_cli_sys(&work1, &["log"]);

    lsc_cli_sys(&work1, &["switch-branch", "main"]);
    lsc_cli_sys(&work1, &["merge-branch", "task"]); //conflict-free merge
    lsc_cli_sys(&work1, &["local-changes"]);
    lsc_cli_sys(&work1, &["commit", r#"-m"merge task branch""#]);
    lsc_cli_sys(&work1, &["log"]);

    //now that task has been merge into main, doing the merge the other way should
    // be a ff merge but for ff detection to work, the previous commit has to
    // have the two parents
    lsc_cli_sys(&work1, &["switch-branch", "task"]);
    lsc_cli_sys(&work1, &["merge-branch", "main"]); //fast-forward

    //making a conflict that can be merged automatically
    lsc_cli_sys(&work1, &["switch-branch", "main"]);
    lsc_cli_sys(&work1, &["edit", "file3.txt"]);
    append_text_to_file(&work1.join("file3.txt"), "line3\n");
    lsc_cli_sys(&work1, &["commit", r#"-m"append line3 to file3""#]);
    lsc_cli_sys(&work1, &["switch-branch", "task"]);
    lsc_cli_sys(&work1, &["edit", "file3.txt"]);
    append_text_to_file(&work1.join("file3.txt"), "line3\nline4\n");
    lsc_cli_sys(&work1, &["commit", r#"-m"append line3,line4 to file3""#]);
    lsc_cli_sys_fail(&work1, &["merge-branch", "main"]); //conflict in file3
    lsc_cli_sys(&work1, &["resolve", "--notool", "file3.txt"]);
    lsc_cli_sys(&work1, &["commit", r#"-m"merge main in task""#]);

    lsc_cli_sys(&work1, &["switch-branch", "main"]);
    lsc_cli_sys(&work1, &["merge-branch", "task"]); //fast-forward

    //lsc_cli_sys(&work1, &["log"]);
    flush_log_buffer();
    flush_metrics_buffer();
    flush_thread_buffer();
}

#[test]
fn test_locks() {
    let telemetry_guard = TelemetryGuard::new();
    std::mem::forget(telemetry_guard);
    trace_scope!("lsc::test_locks");
    let test_dir = init_test_dir("test_locks");
    let config_file_path = lgn_source_control::Config::config_file_path().unwrap();
    if config_file_path.exists() {
        lsc_cli_sys(&test_dir, &["config"]);
    } else {
        println!("no config file, skipping test");
    }

    let work1 = test_dir.join("work1");
    let work2 = test_dir.join("work2");
    let repo_uri = init_test_repo(&test_dir, "test_locks");

    lsc_cli_sys(
        &test_dir,
        &["init-workspace", work1.to_str().unwrap(), &repo_uri],
    );

    std::fs::create_dir_all(work1.join("dir/deep")).unwrap();
    write_file(&work1.join("dir/deep/file1.txt"), b"line1\n").unwrap();

    lsc_cli_sys(&work1, &["lock", "dir\\deep\\file1.txt"]);
    lsc_cli_sys_fail(&work1, &["lock", "dir\\deep\\file1.txt"]);
    lsc_cli_sys_fail(&work1, &["lock", "dir/deep/file1.txt"]);
    lsc_cli_sys(&work1, &["list-locks"]);
    lsc_cli_sys(&work1, &["add", "dir/deep/file1.txt"]);
    lsc_cli_sys(&work1, &["commit", r#"-m"add file1 in task main""#]);

    lsc_cli_sys(&work1, &["create-branch", "task"]);
    lsc_cli_sys_fail(&work1, &["add", "dir/deep/file1.txt"]);
    lsc_cli_sys(&work1, &["unlock", "dir/deep/file1.txt"]);
    lsc_cli_sys(&work1, &["edit", "dir/deep/file1.txt"]);
    lsc_cli_sys(&work1, &["commit", r#"-m"non-edit file1 in task branch""#]);

    lsc_cli_sys(&work1, &["switch-branch", "main"]);
    write_file(&work1.join("file2.txt"), b"line1\n").unwrap();
    lsc_cli_sys(&work1, &["add", "file2.txt"]);
    lsc_cli_sys(&work1, &["commit", r#"-m"add file2 in task main""#]);
    lsc_cli_sys(&work1, &["switch-branch", "task"]);
    lsc_cli_sys(&work1, &["lock", "file2.txt"]); //should it matter that it does not exist here?
    lsc_cli_sys(&work1, &["switch-branch", "main"]);
    lsc_cli_sys_fail(&work1, &["edit", "file2.txt"]); //locked
    lsc_cli_sys(&work1, &["unlock", "file2.txt"]);
    lsc_cli_sys(&work1, &["edit", "file2.txt"]);

    lsc_cli_sys(
        &test_dir,
        &["init-workspace", work2.to_str().unwrap(), &repo_uri],
    );
    lsc_cli_sys(&work2, &["lock", "file2.txt"]); //locking the file that is being edited in work1
    lsc_cli_sys_fail(
        &work1,
        &[
            "commit",
            r#"-m"commiting with file locked in other workspace""#,
        ],
    );
    lsc_cli_sys(&work1, &["unlock", "file2.txt"]);
    lsc_cli_sys(
        &work1,
        &["commit", r#"-m"commiting with file now unlocked""#],
    );

    lsc_cli_sys(&work1, &["lock", "file2.txt"]);
    lsc_cli_sys(&work2, &["sync"]);
    lsc_cli_sys_fail(&work2, &["delete", "file2.txt"]);
    lsc_cli_sys(&work1, &["unlock", "file2.txt"]);
    lsc_cli_sys(&work2, &["delete", "file2.txt"]);
    lsc_cli_sys(&work1, &["lock", "file2.txt"]);
    lsc_cli_sys_fail(
        &work2,
        &[
            "commit",
            r#"-m"commiting with file locked in other workspace""#,
        ],
    );
    lsc_cli_sys(&work1, &["unlock", "file2.txt"]);
    lsc_cli_sys(
        &work2,
        &[
            "commit",
            r#"-m"commiting with file locked in other workspace""#,
        ],
    );

    //create branch hierarchy to test detach
    lsc_cli_sys(&work1, &["create-branch", "feature"]);
    lsc_cli_sys(&work1, &["create-branch", "feature-child"]);
    lsc_cli_sys(&work1, &["create-branch", "feature-gchild"]);
    lsc_cli_sys(&work1, &["lock", "file2.txt"]);
    lsc_cli_sys(&work1, &["switch-branch", "feature-child"]);
    lsc_cli_sys(&work1, &["create-branch", "feature-gchild2"]);
    lsc_cli_sys(&work1, &["switch-branch", "feature"]);
    lsc_cli_sys(&work1, &["detach-branch"]);

    lsc_cli_sys(&work1, &["switch-branch", "main"]);
    lsc_cli_sys(&work1, &["lock", "file2.txt"]);

    lsc_cli_sys(&work1, &["switch-branch", "feature-child"]);
    lsc_cli_sys_fail(&work1, &["attach-branch", "main"]); //branch already has a parent

    lsc_cli_sys(&work1, &["switch-branch", "feature"]);
    lsc_cli_sys_fail(&work1, &["attach-branch", "main"]); //lock domains conflict on file2.txt
    lsc_cli_sys(&work1, &["unlock", "file2.txt"]);
    lsc_cli_sys(&work1, &["lock", "some_other_file.txt"]);
    lsc_cli_sys(&work1, &["attach-branch", "main"]);
    flush_log_buffer();
    flush_metrics_buffer();
    flush_thread_buffer();
}

#[trace_function]
fn get_root_git_directory() -> PathBuf {
    let output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .expect("failed to execute git command");
    PathBuf::from(std::str::from_utf8(&output.stdout).unwrap().trim_end())
}

#[test]
#[ignore] //fails in the build actions because tests don't run under a full git clone, see https://github.com/legion-labs/legion/issues/4
fn test_import_git() {
    let telemetry_guard = TelemetryGuard::new();
    std::mem::forget(telemetry_guard);
    trace_scope!("lsc::test_import_git");
    let test_dir = init_test_dir("test_import_git");
    let work1 = test_dir.join("work1");
    let repo_uri = init_test_repo(&test_dir, "test_import_git");

    lsc_cli_sys(
        &test_dir,
        &["init-workspace", work1.to_str().unwrap(), &repo_uri],
    );
    let root_dir = get_root_git_directory();
    assert!(root_dir.exists());

    lsc_cli_sys(
        &work1,
        &["import-git-branch", root_dir.to_str().unwrap(), "main"],
    );
    flush_log_buffer();
    flush_metrics_buffer();
    flush_thread_buffer();
}
