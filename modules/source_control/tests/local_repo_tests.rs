use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

//
// Tests should run in succession, not in parallel, because they set the current directory.
// cargo test -- --nocapture --test-threads=1
//

fn write_lorem_ipsum(p: &Path) {
    let contents = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. In iaculis odio ac nulla porta, eget dictum nulla euismod. Vivamus congue eros vitae velit feugiat lacinia. Curabitur mi lectus, semper in posuere nec, eleifend eu magna. Morbi egestas magna eget ligula aliquet, vitae mattis urna pellentesque. Maecenas sem risus, scelerisque id semper ut, ornare id diam. Integer ut urna varius, lobortis sapien id, ullamcorper mi. Donec pulvinar ante ligula, in interdum turpis tempor a. Maecenas malesuada turpis orci, vitae efficitur tortor laoreet sit amet.

Nulla eu scelerisque odio. Suspendisse ultrices convallis hendrerit. Duis lacinia lacus ut urna pellentesque, euismod auctor risus volutpat. Sed et congue dolor, et bibendum dolor. Nam sit amet ante id eros aliquet luctus. Donec pulvinar mauris turpis, a ullamcorper mi fermentum ac. Morbi a volutpat turpis. Nulla facilisi. Sed rutrum placerat nisl vitae condimentum. Nunc et lacus ut lacus aliquet tempor et volutpat mi. Maecenas pretium ultricies mi id vestibulum. Sed turpis justo, semper eu nisl ac, hendrerit mattis turpis. Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Praesent condimentum pellentesque vestibulum. Fusce at hendrerit lorem.\n";

    lsc_lib::write_file(p, contents.as_bytes()).expect("write failed");
}

fn append_text_to_file(p: &Path, contents: &str) {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(p)
        .unwrap();
    f.write(contents.as_bytes()).unwrap();
}

fn syscall(command: &str, wd: &Path, args: &[&str], should_succeed: bool) {
    print!("{} ", command);
    for a in args {
        print!("{} ", a);
    }
    print!("\n");
    let status = Command::new(command)
        .current_dir(wd)
        .args(args)
        .status()
        .expect("failed to execute command");

    assert!(status.success() == should_succeed);
}

static LSC_CLI_EXE_VAR: &str = env!("CARGO_BIN_EXE_lsc-cli");
fn lsc_cli_sys(wd: &Path, args: &[&str]) {
    syscall(LSC_CLI_EXE_VAR, wd, args, true);
}

fn lsc_cli_sys_fail(wd: &Path, args: &[&str]) {
    syscall(LSC_CLI_EXE_VAR, wd, args, false);
}

#[test]
fn local_repo_suite() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_dir = temp_dir.path().join("repo");
    let work1 = temp_dir.path().join("work");

    lsc_cli_sys(
        temp_dir.path(),
        &["init-local-repository", repo_dir.to_str().unwrap()],
    );

    lsc_cli_sys(
        temp_dir.path(),
        &[
            "init-workspace",
            work1.to_str().unwrap(),
            repo_dir.to_str().unwrap(),
        ],
    );

    std::fs::create_dir_all(work1.join("dir0/deep")).expect("dir0 creation failed");
    write_lorem_ipsum(&work1.join("dir0/file0.txt"));
    write_lorem_ipsum(&work1.join("dir0/file1.txt"));
    write_lorem_ipsum(&work1.join("dir0/deep/file2.txt"));
    std::fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/lambda.jpg"),
        work1.join("bin.jpg"),
    )
    .expect("error copying lambda.jpg");

    lsc_cli_sys(
        work1.as_path(),
        &["add", work1.join("dir0/file0.txt").to_str().unwrap()],
    );

    lsc_cli_sys(work1.as_path(), &["add", "dir0/file1.txt"]);
    lsc_cli_sys(work1.as_path(), &["add", "dir0/deep/file2.txt"]);
    lsc_cli_sys(work1.as_path(), &["add", "bin.jpg"]);
    lsc_cli_sys(work1.as_path(), &["local-changes"]);
    lsc_cli_sys(work1.as_path(), &["commit", r#"-m"my commit message""#]);

    write_lorem_ipsum(&work1.join("dir0/file3.txt"));
    lsc_cli_sys(work1.as_path(), &["add", "dir0/file3.txt"]);
    lsc_cli_sys(work1.as_path(), &["local-changes"]);
    lsc_cli_sys(
        work1.as_path(),
        &["commit", r#"-m"my second commit message""#],
    );

    let work2 = temp_dir.path().join("work2");
    lsc_cli_sys(
        temp_dir.path(),
        &[
            "init-workspace",
            work2.to_str().unwrap(),
            repo_dir.to_str().unwrap(),
        ],
    );
    assert!(fs::metadata(work2.join("dir0/file3.txt")).is_ok());
    assert!(fs::metadata(work2.join("dir0/file1.txt")).is_ok());

    //still under first workspace
    lsc_cli_sys(work1.as_path(), &["log"]);
    lsc_cli_sys(work1.as_path(), &["edit", "dir0/file0.txt"]);
    lsc_cli_sys(work1.as_path(), &["local-changes"]);
    append_text_to_file(&work1.join("dir0/file0.txt"), "\nnew line in file0");
    lsc_cli_sys(work1.as_path(), &["diff", "--notool", "dir0/file0.txt"]);
    lsc_cli_sys(
        work1.as_path(),
        &["diff", "--notool", "dir0/file0.txt", "latest"],
    );
    lsc_cli_sys(work1.as_path(), &["commit", r#"-m"edit file0""#]);

    lsc_cli_sys(work1.as_path(), &["edit", "dir0/file1.txt"]);
    append_text_to_file(&work1.join("dir0/file1.txt"), "\nnew line in file1");
    lsc_cli_sys(work1.as_path(), &["commit", r#"-m"edit file1""#]);

    lsc_cli_sys(work2.as_path(), &["log"]);

    //should not be allowed to commit before sync
    lsc_cli_sys(work2.as_path(), &["edit", "dir0/file0.txt"]);
    lsc_cli_sys_fail(work2.as_path(), &["commit", r#"-m"bad commit""#]);
    lsc_cli_sys(work2.as_path(), &["revert", "dir0/file0.txt"]);
    lsc_cli_sys(work2.as_path(), &["sync"]);

    lsc_cli_sys(work2.as_path(), &["delete", "dir0/file0.txt"]);
    lsc_cli_sys(work2.as_path(), &["local-changes"]);
    lsc_cli_sys(work2.as_path(), &["log"]);
    lsc_cli_sys(work2.as_path(), &["commit", r#"-m"delete file0""#]);

    // Switching back to wd1
    lsc_cli_sys(work1.as_path(), &["log"]);
    lsc_cli_sys(work1.as_path(), &["sync"]);

    //test revert add
    write_lorem_ipsum(&work1.join("dir0/added_and_reverted.txt"));
    lsc_cli_sys(work1.as_path(), &["add", "dir0/added_and_reverted.txt"]);
    lsc_cli_sys(work1.as_path(), &["revert", "dir0/added_and_reverted.txt"]);

    //test revert delete
    lsc_cli_sys(work1.as_path(), &["delete", "dir0/file1.txt"]);
    lsc_cli_sys(work1.as_path(), &["revert", "dir0/file1.txt"]);

    //sync backwards
    let log_vec = lsc_lib::find_branch_commits(&work1).unwrap();
    let init_commit = log_vec.last().unwrap();
    lsc_cli_sys(work1.as_path(), &["sync", &init_commit.id]);

    //sync forwards
    lsc_cli_sys(work1.as_path(), &["sync"]);
}

#[test]
fn local_single_branch_merge_flow() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_dir = temp_dir.path().join("repo");
    let work1 = temp_dir.path().join("work1");
    let work2 = temp_dir.path().join("work2");

    lsc_cli_sys(
        temp_dir.path(),
        &["init-local-repository", repo_dir.to_str().unwrap()],
    );

    lsc_cli_sys(
        temp_dir.path(),
        &[
            "init-workspace",
            work1.to_str().unwrap(),
            repo_dir.to_str().unwrap(),
        ],
    );

    lsc_lib::write_file(&work1.join("file1.txt"), "line1\n".as_bytes()).unwrap();
    lsc_cli_sys(work1.as_path(), &["add", "file1.txt"]);
    lsc_cli_sys(work1.as_path(), &["commit", r#"-m"add file1""#]);

    lsc_cli_sys(
        temp_dir.path(),
        &[
            "init-workspace",
            work2.to_str().unwrap(),
            repo_dir.to_str().unwrap(),
        ],
    );

    lsc_cli_sys(work2.as_path(), &["edit", "file1.txt"]);
    append_text_to_file(&work2.join("file1.txt"), "line2\n");
    lsc_cli_sys(work2.as_path(), &["commit", r#"-m"add line2 to file1""#]);

    lsc_cli_sys(work1.as_path(), &["edit", "file1.txt"]);
    append_text_to_file(&work1.join("file1.txt"), "line2\nline3\n");
    lsc_cli_sys_fail(
        work1.as_path(),
        &["commit", r#"-m"should fail - not at head""#],
    );
    lsc_cli_sys(work1.as_path(), &["sync"]);
    lsc_cli_sys(work1.as_path(), &["merges-pending"]);
    lsc_cli_sys(work1.as_path(), &["merge", "file1.txt"]);
    lsc_cli_sys(work1.as_path(), &["merges-pending"]);
    lsc_cli_sys(work1.as_path(), &["commit", r#"-m"merged""#]);
}

#[test]
fn test_print_config() {
    let config_file_path = lsc_lib::Config::config_file_path().unwrap();
    if config_file_path.exists() {
        lsc_cli_sys(std::env::current_dir().unwrap().as_path(), &["config"]);
    } else {
        println!("no config file, skipping test");
    }
}

#[test]
fn test_branch() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_file_path = lsc_lib::Config::config_file_path().unwrap();
    if config_file_path.exists() {
        lsc_cli_sys(temp_dir.path(), &["config"]);
    } else {
        println!("no config file, skipping test");
    }

    let repo_dir = temp_dir.path().join("repo");
    let work1 = temp_dir.path().join("work1");

    lsc_cli_sys(
        temp_dir.path(),
        &["init-local-repository", repo_dir.to_str().unwrap()],
    );

    lsc_cli_sys(
        temp_dir.path(),
        &[
            "init-workspace",
            work1.to_str().unwrap(),
            repo_dir.to_str().unwrap(),
        ],
    );

    assert!(std::env::set_current_dir(&work1).is_ok());
    lsc_lib::write_file(&work1.join("file1.txt"), "line1\n".as_bytes()).unwrap();
    lsc_cli_sys(work1.as_path(), &["add", "file1.txt"]);

    lsc_lib::write_file(&work1.join("file2.txt"), "line1\n".as_bytes()).unwrap();
    lsc_cli_sys(work1.as_path(), &["add", "file2.txt"]);

    lsc_cli_sys(work1.as_path(), &["commit", r#"-m"add file1""#]);
    lsc_cli_sys(work1.as_path(), &["create-branch", "task"]);
    lsc_cli_sys(work1.as_path(), &["edit", "file1.txt"]);
    append_text_to_file(&work1.join("file1.txt"), "\nfrom task branch");

    lsc_cli_sys(work1.as_path(), &["delete", "file2.txt"]);

    lsc_lib::write_file(&work1.join("file3.txt"), "line1\n".as_bytes()).unwrap();
    lsc_cli_sys(work1.as_path(), &["add", "file3.txt"]);

    std::fs::create_dir_all(work1.join("dir0/deep")).expect("dir0 creation failed");
    lsc_lib::write_file(
        &work1.join("dir0/deep/inner_task.txt"),
        "line1\n".as_bytes(),
    )
    .unwrap();
    lsc_cli_sys(work1.as_path(), &["add", "dir0/deep/inner_task.txt"]);

    lsc_cli_sys(work1.as_path(), &["commit", r#"-m"task complete""#]);
    lsc_cli_sys(work1.as_path(), &["log"]);
    lsc_cli_sys(work1.as_path(), &["switch-branch", "main"]);
    lsc_cli_sys(work1.as_path(), &["log"]);

    lsc_cli_sys(work1.as_path(), &["switch-branch", "task"]);
    lsc_cli_sys(work1.as_path(), &["log"]);
}
