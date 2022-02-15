use std::process::Command;

static EDITOR_SERVER_EXE: &str = env!("CARGO_BIN_EXE_editor-srv");

#[test]
fn lifecycle_test() {
    let args = &["--test", "lifecycle"];
    println!("{} {}", EDITOR_SERVER_EXE, args.join(" "));
    let status = Command::new(EDITOR_SERVER_EXE)
        .args(args)
        .envs(std::env::vars())
        .status()
        .expect("failed to execute command");
    assert!(status.success());
}
