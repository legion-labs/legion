mod lsc_lib;
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
                        .short("r")
                        .value_name("repository-directory")
                        .required(true)
                        .help("lsc database directory"),
                ),
        )
        .subcommand(
            SubCommand::with_name("init-workspace")
                .about("Initializes a workspace and populates it with the latest version of the main branch")
                .arg(
                    Arg::with_name("workspace-directory")
                        .short("w")
                        .value_name("workspace-directory")
                        .required(true)
                        .help("lsc workspace directory"))
                .arg(
                    Arg::with_name("repository-directory")
                        .short("r")
                        .value_name("repository-directory")
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
        .get_matches();

    match matches.subcommand() {
        ("init-local-repository", Some(command_match)) => {
            if let Err(e) = lsc_lib::init_local_repository(
                command_match.value_of("repository-directory").unwrap(),
            ) {
                println!("init_local_repository failed: {}", e);
                std::process::exit(1);
            }
        }
        ("init-workspace", Some(command_match)) => {
            if let Err(e) = init_workspace(
                Path::new(command_match.value_of("workspace-directory").unwrap()),
                Path::new(command_match.value_of("repository-directory").unwrap()),
            ) {
                println!("init_workspace failed: {}", e);
                std::process::exit(1);
            }
        }
        ("add", Some(command_match)) => {
            if let Err(e) = track_new_file(Path::new(command_match.value_of("path").unwrap())) {
                println!("add failed: {}", e);
                std::process::exit(1);
            }
        }
        _ => {}
    }
}
