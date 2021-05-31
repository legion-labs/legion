use clap::{App, AppSettings, Arg, SubCommand};
use lsc_lib::*;
use std::path::Path;

fn main() {
    let matches = App::new("Legion Source Control")
        .version("0.1.0")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("init-local-repository")
                .about("Initializes a repository stored on a local filesystem")
                .arg(
                    Arg::with_name("repository-directory")
                        .required(true)
                        .help("lsc database directory"),
                ),
        )
        .subcommand(
            SubCommand::with_name("init-workspace")
                .about("Initializes a workspace and populates it with the latest version of the main branch")
                .arg(
                    Arg::with_name("workspace-directory")
                        .required(true)
                        .help("lsc workspace directory"))
                .arg(
                    Arg::with_name("repository-directory")
                        .required(true)
                        .help("local repository directory"),
                ),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Adds local file to the set of pending changes")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("local path within a workspace")),
        )
        .subcommand(
            SubCommand::with_name("edit")
                .about("Makes file writable and adds it to the set of pending changes")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("local path within a workspace")),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .about("Deletes the local file and records the pending change")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("local path within a workspace")),
        )
        .subcommand(
            SubCommand::with_name("revert")
                .about("Abandon the local changes made to a file. Overwrites the content of the file based on the current commit.")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("local path within a workspace")),
        )
        .subcommand(
            SubCommand::with_name("local-changes")
                .about("Lists changes in workspace lsc knows about")
        )
        .subcommand(
            SubCommand::with_name("log")
                .about("Lists commits of the current branch")
        )
        .subcommand(
            SubCommand::with_name("sync")
                .about("Updates the workspace with the latest version of the files")
        )
        .subcommand(
            SubCommand::with_name("commit")
                .about("Records local changes in the repository as a single transaction")
                .arg(
                    Arg::with_name("message")
                        .short("m")
                        .required(true)
                        .value_delimiter("\"")
                        .help("commit message"))
        )
        .get_matches();

    match matches.subcommand() {
        ("init-local-repository", Some(command_match)) => {
            match lsc_lib::init_local_repository(Path::new(
                command_match.value_of("repository-directory").unwrap(),
            )) {
                Err(e) => {
                    println!("init_local_repository failed: {}", e);
                    std::process::exit(1);
                }
                Ok(_) => {
                    println!("repository initialized");
                }
            }
        }
        ("init-workspace", Some(command_match)) => {
            match init_workspace(
                Path::new(command_match.value_of("workspace-directory").unwrap()),
                Path::new(command_match.value_of("repository-directory").unwrap()),
            ) {
                Err(e) => {
                    println!("init_workspace failed: {}", e);
                    std::process::exit(1);
                }
                Ok(_) => {
                    println!("workspace initialized");
                }
            }
        }
        ("add", Some(command_match)) => {
            match track_new_file(Path::new(command_match.value_of("path").unwrap())) {
                Err(e) => {
                    println!("add failed: {}", e);
                    std::process::exit(1);
                }
                Ok(_) => {
                    println!("tracking new file");
                }
            }
        }
        ("edit", Some(command_match)) => {
            if let Err(e) = edit_file_command(Path::new(command_match.value_of("path").unwrap())) {
                println!("edit failed: {}", e);
                std::process::exit(1);
            } else {
                println!("file ready to be edited");
            }
        }
        ("delete", Some(command_match)) => {
            if let Err(e) = delete_file_command(Path::new(command_match.value_of("path").unwrap())) {
                println!("delete failed: {}", e);
                std::process::exit(1);
            } else {
                println!("file deleted, pending change recorded");
            }
        }
        ("revert", Some(command_match)) => {
            if let Err(e) = revert_file_command(Path::new(command_match.value_of("path").unwrap())) {
                println!("revert failed: {}", e);
                std::process::exit(1);
            } else {
                println!("file reverted");
            }
        }
        ("commit", Some(command_match)) => {
            let mut message = String::from("");
            for item in command_match.values_of("message").unwrap() {
                message += item;
            }
            match commit(&message) {
                Err(e) => {
                    println!("commit failed: {}", e);
                    std::process::exit(1);
                }
                Ok(_) => {
                    println!("commit completed");
                }
            }
        }
        ("local-changes", Some(_command_match)) => match find_local_changes_command() {
            Ok(changes) => {
                if changes.is_empty(){
                    println!("No local changes");
                }
                for change in changes {
                    println!("{} {}", change.change_type, change.relative_path.display());
                }
            }
            Err(e) => {
                println!("local-changes failed: {}", e);
                std::process::exit(1);
            }
        },
        ("sync", Some(_command_match)) => match sync_command() {
            Ok(_) => {
                println!("sync completed");
            }
            Err(e) => {
                println!("sync failed: {}", e);
                std::process::exit(1);
            }
        },
        ("log", Some(_command_match)) => {
            if let Err(e) = log_command(){
                println!("{}", e);
                std::process::exit(1);
            }
        },
        _ => {}
    }
}
