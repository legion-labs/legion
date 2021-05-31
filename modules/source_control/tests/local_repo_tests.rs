use std::fs::{self, DirEntry};
use std::io::{self, Write};
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

fn syscall(command: &str, args: &[&str], should_succeed: bool) {
    print!("{} ", command);
    for a in args {
        print!("{} ", a);
    }
    print!("\n");
    let output = Command::new(command)
        .args(args)
        .output()
        .expect("failed to execute lsc-cli");

    let mut out = std::io::stdout();
    out.write_all(&output.stdout).unwrap();
    out.flush().unwrap();

    let mut err = std::io::stderr();
    err.write_all(&output.stderr).unwrap();
    err.flush().unwrap();

    assert!(output.status.success() == should_succeed);
}

fn lsc_cli_sys_impl(args: &[&str], should_succeed: bool) {
    let test_bin_exe = std::env::current_exe().expect("error getting current exe");
    let mut subdir = test_bin_exe
        .parent()
        .expect("error getting parent directory");
    let mut parent_dir = subdir.parent().expect("error getting parent directory");
    while parent_dir.file_name().expect("file name") != "target" {
        subdir = parent_dir;
        parent_dir = subdir.parent().expect("error getting parent directory");
    }

    let command = subdir.join("lsc-cli");

    syscall(
        command.to_str().expect("command path"),
        args,
        should_succeed,
    );
}

fn lsc_cli_sys(args: &[&str]) {
    lsc_cli_sys_impl(&args, true);
}

fn lsc_cli_sys_fail(args: &[&str]) {
    lsc_cli_sys_impl(&args, false);
}

fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
                cb(&entry);
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn force_delete_all(dir: &Path) {
    //std::fs::remove_dir_all leaves read-only files and reports an error
    visit_dirs(dir, &|entry| {
        let p = entry.path();
        let meta = entry.metadata().unwrap();
        if meta.is_dir() {
            fs::remove_dir(p).unwrap();
        } else {
            let mut perm = meta.permissions();
            if perm.readonly() {
                perm.set_readonly(false);
                fs::set_permissions(&p, perm).unwrap();
            }
            fs::remove_file(&p).unwrap();
        }
    })
    .unwrap();
}

#[test]
fn local_repo_suite() {
    let cargo_project_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let test_scratch_dir = cargo_project_dir.join("target/test_scratch");
    let this_test_dir = test_scratch_dir.join("local_repo_suite");

    if let Ok(_) = std::fs::metadata(&this_test_dir) {
        force_delete_all(&this_test_dir);
    }
    std::fs::create_dir_all(&this_test_dir).unwrap();

    let repo_dir = this_test_dir.join("repo");
    let work1 = this_test_dir.join("work");

    lsc_cli_sys(&["init-local-repository", repo_dir.to_str().unwrap()]);

    lsc_cli_sys(&[
        "init-workspace",
        work1.to_str().unwrap(),
        repo_dir.to_str().unwrap(),
    ]);

    std::fs::create_dir_all(work1.join("dir0/deep")).expect("dir0 creation failed");
    write_lorem_ipsum(&work1.join("dir0/file0.txt"));
    write_lorem_ipsum(&work1.join("dir0/file1.txt"));
    write_lorem_ipsum(&work1.join("dir0/deep/file2.txt"));
    std::fs::copy(
        cargo_project_dir.join("tests/lambda.jpg"),
        work1.join("bin.jpg"),
    )
    .expect("error copying lambda.jpg");

    lsc_cli_sys(&["add", work1.join("dir0/file0.txt").to_str().unwrap()]);

    assert!(std::env::set_current_dir(&work1).is_ok());
    lsc_cli_sys(&["add", "dir0/file1.txt"]);
    lsc_cli_sys(&["add", "dir0/deep/file2.txt"]);
    lsc_cli_sys(&["add", "bin.jpg"]);
    lsc_cli_sys(&["local-changes"]);
    lsc_cli_sys(&["commit", r#"-m"my commit message""#]);

    write_lorem_ipsum(&work1.join("dir0/file3.txt"));
    lsc_cli_sys(&["add", "dir0/file3.txt"]);
    lsc_cli_sys(&["local-changes"]);
    lsc_cli_sys(&["commit", r#"-m"my second commit message""#]);

    let work2 = this_test_dir.join("work2");
    lsc_cli_sys(&[
        "init-workspace",
        work2.to_str().unwrap(),
        repo_dir.to_str().unwrap(),
    ]);

    assert!(fs::metadata(work2.join("dir0/file3.txt")).is_ok());
    assert!(fs::metadata(work2.join("dir0/file1.txt")).is_ok());

    //still under first workspace
    lsc_cli_sys(&["log"]);
    lsc_cli_sys(&["edit", "dir0/file0.txt"]);
    lsc_cli_sys(&["local-changes"]);
    append_text_to_file(Path::new("dir0/file0.txt"), "\nnew line in file0");
    lsc_cli_sys(&["diff", "--notool", "dir0/file0.txt"]);
    lsc_cli_sys(&["diff", "--notool", "dir0/file0.txt", "latest"]);
    lsc_cli_sys(&["commit", r#"-m"edit file0""#]);

    lsc_cli_sys(&["edit", "dir0/file1.txt"]);
    append_text_to_file(Path::new("dir0/file1.txt"), "\nnew line in file1");
    lsc_cli_sys(&["commit", r#"-m"edit file1""#]);

    assert!(std::env::set_current_dir(&work2).is_ok());
    lsc_cli_sys(&["log"]);

    //should not be allowed to commit before sync
    lsc_cli_sys(&["edit", "dir0/file0.txt"]);
    lsc_cli_sys_fail(&["commit", r#"-m"bad commit""#]);
    lsc_cli_sys(&["revert", "dir0/file0.txt"]);
    lsc_cli_sys(&["sync"]);

    lsc_cli_sys(&["delete", "dir0/file0.txt"]);
    lsc_cli_sys(&["local-changes"]);
    lsc_cli_sys(&["log"]);
    lsc_cli_sys(&["commit", r#"-m"delete file0""#]);

    assert!(std::env::set_current_dir(&work1).is_ok());
    lsc_cli_sys(&["log"]);
    lsc_cli_sys(&["sync"]);

    //test revert add
    write_lorem_ipsum(&work1.join("dir0/added_and_reverted.txt"));
    lsc_cli_sys(&["add", "dir0/added_and_reverted.txt"]);
    lsc_cli_sys(&["revert", "dir0/added_and_reverted.txt"]);

    //test revert delete
    lsc_cli_sys(&["delete", "dir0/file1.txt"]);
    lsc_cli_sys(&["revert", "dir0/file1.txt"]);

    //sync backwards
    let log_vec = lsc_lib::find_branch_commits(&work1).unwrap();
    let init_commit = log_vec.last().unwrap();
    lsc_cli_sys(&["sync", &init_commit.id]);

    //sync forwards
    lsc_cli_sys(&["sync"]);
}

#[test]
fn local_single_branch_merge_flow() {
    let cargo_project_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let test_scratch_dir = cargo_project_dir.join("target/test_scratch");
    let this_test_dir = test_scratch_dir.join("single_merge");
    if let Ok(_) = std::fs::metadata(&this_test_dir) {
        force_delete_all(&this_test_dir);
    }
    std::fs::create_dir_all(&this_test_dir).unwrap();
    let repo_dir = this_test_dir.join("repo");
    let work1 = this_test_dir.join("work1");
    let work2 = this_test_dir.join("work2");

    lsc_cli_sys(&["init-local-repository", repo_dir.to_str().unwrap()]);

    lsc_cli_sys(&[
        "init-workspace",
        work1.to_str().unwrap(),
        repo_dir.to_str().unwrap(),
    ]);

    assert!(std::env::set_current_dir(&work1).is_ok());
    lsc_lib::write_file(Path::new("file1.txt"), "line1\n".as_bytes()).unwrap();
    lsc_cli_sys(&["add", "file1.txt"]);
    lsc_cli_sys(&["commit", r#"-m"add file1""#]);

    lsc_cli_sys(&[
        "init-workspace",
        work2.to_str().unwrap(),
        repo_dir.to_str().unwrap(),
    ]);
    assert!(std::env::set_current_dir(&work2).is_ok());
    lsc_cli_sys(&["edit", "file1.txt"]);
    append_text_to_file(Path::new("file1.txt"), "line2\n");
    lsc_cli_sys(&["commit", r#"-m"add line2 to file1""#]);

    assert!(std::env::set_current_dir(&work1).is_ok());
    lsc_cli_sys(&["edit", "file1.txt"]);
    append_text_to_file(Path::new("file1.txt"), "line2\nline3\n");
    lsc_cli_sys_fail(&["commit", r#"-m"should fail - not at head""#]);
    lsc_cli_sys(&["sync"]);
    lsc_cli_sys(&["merges-pending"]);
    lsc_cli_sys(&["merge", "file1.txt"]);
    lsc_cli_sys(&["merges-pending"]);
    lsc_cli_sys(&["commit", r#"-m"merged""#]);
}

#[test]
fn test_print_config() {
    let config_file_path = lsc_lib::Config::config_file_path().unwrap();
    if config_file_path.exists() {
        lsc_cli_sys(&["config"]);
    } else {
        println!("no config file, skipping test");
    }
}
