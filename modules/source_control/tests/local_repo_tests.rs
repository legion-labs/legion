use std::path::{Path, PathBuf};
use std::process::Command;

fn write_lorem_ipsum(p: &Path) {
    let contents = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. In iaculis odio ac nulla porta, eget dictum nulla euismod. Vivamus congue eros vitae velit feugiat lacinia. Curabitur mi lectus, semper in posuere nec, eleifend eu magna. Morbi egestas magna eget ligula aliquet, vitae mattis urna pellentesque. Maecenas sem risus, scelerisque id semper ut, ornare id diam. Integer ut urna varius, lobortis sapien id, ullamcorper mi. Donec pulvinar ante ligula, in interdum turpis tempor a. Maecenas malesuada turpis orci, vitae efficitur tortor laoreet sit amet.

Nulla eu scelerisque odio. Suspendisse ultrices convallis hendrerit. Duis lacinia lacus ut urna pellentesque, euismod auctor risus volutpat. Sed et congue dolor, et bibendum dolor. Nam sit amet ante id eros aliquet luctus. Donec pulvinar mauris turpis, a ullamcorper mi fermentum ac. Morbi a volutpat turpis. Nulla facilisi. Sed rutrum placerat nisl vitae condimentum. Nunc et lacus ut lacus aliquet tempor et volutpat mi. Maecenas pretium ultricies mi id vestibulum. Sed turpis justo, semper eu nisl ac, hendrerit mattis turpis. Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Praesent condimentum pellentesque vestibulum. Fusce at hendrerit lorem.";

    lsc_lib::write_file(p, contents.as_bytes()).expect("write failed");
}

fn syscall(command: &str, args: &[&str]) {
    print!("{} ", command);
    for a in args{
        print!("{} ", a);
    }
    print!("\n");
    let output = Command::new(command)
        .args(args)
        .output()
        .expect("failed to execute lsc-cli");
    println!("{}", std::str::from_utf8(&output.stdout).unwrap());
    println!("{}", std::str::from_utf8(&output.stderr).unwrap());
    assert!(output.status.success());
}

fn lsc_cli_sys(args: &[&str]) {
    syscall("lsc-cli", args);
}

#[test]
fn add_files() {
    let cargo_project_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let test_scratch_dir = cargo_project_dir.join("target/test_scratch");
    let this_test_dir = test_scratch_dir.join("add_files");

    if let Ok(_) = std::fs::metadata(&this_test_dir) {
        std::fs::remove_dir_all(&this_test_dir).unwrap();
    }
    std::fs::create_dir_all(&this_test_dir).unwrap();

    let repo_dir = this_test_dir.join("repo");
    let workspace_dir = this_test_dir.join("work");

    lsc_cli_sys(&["init-local-repository", repo_dir.to_str().unwrap()]);

    lsc_cli_sys(&[
        "init-workspace",
        workspace_dir.to_str().unwrap(),
        repo_dir.to_str().unwrap(),
    ]);

    std::fs::create_dir_all(workspace_dir.join("dir0/deep")).expect("dir0 creation failed");
    write_lorem_ipsum(&workspace_dir.join("dir0/file0.txt"));
    write_lorem_ipsum(&workspace_dir.join("dir0/file1.txt"));
    write_lorem_ipsum(&workspace_dir.join("dir0/deep/file2.txt"));
    std::fs::copy( cargo_project_dir.join( "tests/lambda.jpg" ), workspace_dir.join("bin.jpg") ).expect("error copying lambda.jpg");

    lsc_cli_sys(&[
        "add",
        workspace_dir.join("dir0/file0.txt").to_str().unwrap(),
    ]);

    assert!(std::env::set_current_dir(&workspace_dir).is_ok());
    lsc_cli_sys(&["add", "dir0/file1.txt"]);
    lsc_cli_sys(&["add", "dir0/deep/file2.txt"]);
    lsc_cli_sys(&["add", "bin.jpg"]);
    lsc_cli_sys(&["local-changes"]);
    lsc_cli_sys(&["commit", r#"-m"my commit message""#]);
}
