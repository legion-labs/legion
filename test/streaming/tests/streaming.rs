use std::process::Command;

fn run_command(command: &str, args: &[&str]) {
    print!("{} ", command);
    for a in args {
        print!("{} ", a);
    }
    println!();
    let status = Command::new(command)
        .args(args)
        .status()
        .expect("failed to execute command");

    assert!(status.success());
}

static STREAMER_CLIENT_EXE: &str = env!("CARGO_BIN_EXE_streamer-client-test");

static STREAMER_SERVER_EXE: &str = env!("CARGO_BIN_EXE_streamer-server-test");

#[test]
fn streaming_test() {
    let client_handler = std::thread::spawn(|| {
        run_command(STREAMER_CLIENT_EXE, &[]);
    });

    let server_handler = std::thread::spawn(|| {
        run_command(STREAMER_SERVER_EXE, &[]);
    });

    server_handler.join().unwrap();
    client_handler.join().unwrap();
}
