use clap::{App, AppSettings, Arg, SubCommand};
use lsc_lib::*;
use std::path::Path;

fn main() {
    let matches = App::new("Legion Source Control 0.1")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DisableVersion)
        .subcommand(
            SubCommand::with_name("init-local-repository")
                .about("Initializes a repository stored on a local filesystem")
                .arg(
                    Arg::with_name("repository-directory")
                        .required(true)
                        .help("lsc database directory")
                )
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
                        .help("local repository directory")
                )
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Adds local file to the set of pending changes")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("local path within a workspace"))
        )
        .subcommand(
            SubCommand::with_name("edit")
                .about("Makes file writable and adds it to the set of pending changes")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("local path within a workspace"))
        )
        .subcommand(
            SubCommand::with_name("delete")
                .about("Deletes the local file and records the pending change")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("local path within a workspace"))
        )
        .subcommand(
            SubCommand::with_name("diff")
                .about("Prints difference between local file and specified commit")
                .arg(
                    Arg::with_name("notool")
                        .long("notool")
                        .help("ignores diff tool config and prints a patch on stdout"))
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("local path within a workspace"))
                .arg(
                    Arg::with_name("reference")
                        .help("reference version: a commit id, base or latest"))
        )
        .subcommand(
            SubCommand::with_name("merge")
                .about("Reconciles local modifications with colliding changes from other workspaces")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("local path within a workspace"))
        )
        .subcommand(
            SubCommand::with_name("create-branch")
                .about("Creates a new branch based on the state of the workspace")
                .arg(
                    Arg::with_name("name")
                        .required(true)
                        .help("name of the new branch"))
        )
        .subcommand(
            SubCommand::with_name("switch-branch")
                .about("Syncs workspace to specified branch")
                .arg(
                    Arg::with_name("name")
                        .required(true)
                        .help("name of the existing branch to sync to"))
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
                .arg(
                    Arg::with_name("commit-id")
                        .help("version to sync to"))
        )
        .subcommand(
            SubCommand::with_name("merges-pending")
                .about("Lists the files that are scheduled to be merged following a sync with colliding changes")
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
        .subcommand(
            SubCommand::with_name("config")
                .about("Prints the path to the configuration file and its content")
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
            if let Err(e) = delete_file_command(Path::new(command_match.value_of("path").unwrap()))
            {
                println!("delete failed: {}", e);
                std::process::exit(1);
            } else {
                println!("file deleted, pending change recorded");
            }
        }
        ("diff", Some(command_match)) => {
            let notool = command_match.is_present("notool");
            let reference_version_name = command_match.value_of("reference").unwrap_or("base");
            if let Err(e) = diff_file_command(
                Path::new(command_match.value_of("path").unwrap()),
                &reference_version_name,
                !notool,
            ) {
                println!("diff failed: {}", e);
                std::process::exit(1);
            }
        }
        ("merge", Some(command_match)) => {
            if let Err(e) = merge_file_command(Path::new(command_match.value_of("path").unwrap())) {
                println!("merge failed: {}", e);
                std::process::exit(1);
            }
        }
        ("create-branch", Some(command_match)) => {
            let name = command_match.value_of("name").unwrap();
            if let Err(e) = create_branch_command(&name) {
                println!("create branch failed: {}", e);
                std::process::exit(1);
            } else {
                println!("now on branch {}", &name);
            }
        }
        ("switch-branch", Some(command_match)) => {
            let name = command_match.value_of("name").unwrap();
            if let Err(e) = switch_branch_command(&name) {
                println!("switch branch failed: {}", e);
                std::process::exit(1);
            } else {
                println!("now on branch {}", &name);
            }
        }
        ("revert", Some(command_match)) => {
            if let Err(e) = revert_file_command(Path::new(command_match.value_of("path").unwrap()))
            {
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
                if changes.is_empty() {
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
        ("merges-pending", Some(_command_match)) => match find_merges_pending_command() {
            Ok(merges_pending) => {
                if merges_pending.is_empty() {
                    println!("No merges pending");
                }
                for m in merges_pending {
                    println!(
                        "{} {} {}",
                        m.relative_path.display(),
                        &m.base_commit_id,
                        &m.theirs_commit_id
                    );
                }
            }
            Err(e) => {
                println!("merges-pending failed: {}", e);
                std::process::exit(1);
            }
        },
        ("sync", Some(command_match)) => {
            let sync_result;
            match command_match.value_of("commit-id") {
                Some(commit_id) => {
                    sync_result = sync_to_command(&commit_id);
                }
                None => {
                    sync_result = sync_command();
                }
            }
            match sync_result {
                Ok(_) => {
                    println!("sync completed");
                }
                Err(e) => {
                    println!("sync failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        ("log", Some(_command_match)) => {
            if let Err(e) = log_command() {
                println!("{}", e);
                std::process::exit(1);
            }
        }
        ("config", Some(_command_match)) => {
            if let Err(e) = print_config_command() {
                println!("{}", e);
                std::process::exit(1);
            }
        }
        other_match => {
            println!("unknown subcommand match: {:?}", &other_match);
            std::process::exit(1);
        }
    }
}
