//! Source control CLI
//!

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow(clippy::exit, clippy::too_many_lines, clippy::wildcard_imports)]

use std::path::Path;

use clap::{App, AppSettings, Arg, SubCommand};
use legion_source_control::*;
use legion_telemetry::*;

fn main() {
    let _telemetry_guard = TelemetrySystemGuard::new(None);
    let _telemetry_thread_guard = TelemetryThreadGuard::new();
    if let Err(e) = main_impl() {
        println!("{}", e);
        std::process::exit(1);
    }
}

fn main_impl() -> Result<(), String> {
    trace_scope!();
    let matches = App::new("Legion Source Control")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .about("CLI to interact with Legion Source Control")
        .subcommand(
            SubCommand::with_name("init-local-repository")
                .about("Initializes a repository stored on a local or remote system")
                .arg(
                    Arg::with_name("directory")
                        .required(true)
                        .help("local path")
                )
        )
        .subcommand(
            SubCommand::with_name("init-remote-repository")
                .about("Initializes a repository stored on a local or remote system")
                .arg(
                    Arg::with_name("uri")
                        .required(true)
                        .help("mysql://user:pass@host:port/database, lsc://host:port/database")
                )
                .arg(
                    Arg::with_name("blob-storage")
                        .required(false)
                        .help("file://somepath, s3://bucket/root")
                )
        )
        .subcommand(
            SubCommand::with_name("destroy-repository")
                .about("Destroys all repository data permanently")
                .arg(
                    Arg::with_name("uri")
                        .required(true)
                        .help("file://somepath, mysql://user:pass@host:port/database, lsc://host:port/database")
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
                    Arg::with_name("repository-uri")
                        .required(true)
                        .help("uri printed at the creation of the repository"))
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
            SubCommand::with_name("lock")
                .about("Prevent others from modifying the specified file. Locks apply throught all related branches")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("local path within a workspace"))
        )
        .subcommand(
            SubCommand::with_name("unlock")
                .about("Releases a lock, allowing others to modify or lock the file")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("local path within a workspace"))
        )
        .subcommand(
            SubCommand::with_name("list-locks")
                .about("Prints all the locks in the current lock domain")
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
            SubCommand::with_name("resolve")
                .about("Reconciles local modifications with colliding changes from other workspaces")
                .arg(
                    Arg::with_name("notool")
                        .long("notool")
                        .help("ignores merge tool config"))
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
            SubCommand::with_name("merge-branch")
                .about("Merge the specified branch into the current one")
                .arg(
                    Arg::with_name("name")
                        .required(true)
                        .help("name of the branch to merge"))
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
            SubCommand::with_name("detach-branch")
                .about("Move the current branch and its descendance to a new lock domain")

        )
        .subcommand(
            SubCommand::with_name("attach-branch")
                .about("Merges the lock domains of the two branches")
                .arg(
                    Arg::with_name("parent-branch-name")
                        .required(true)
                        .help("name of the existing branch to set as parent"))

        )
        .subcommand(
            SubCommand::with_name("list-branches")
                .about("Prints a list of all branches")
        )
        .subcommand(
            SubCommand::with_name("revert")
                .about("Abandon the local changes made to a file. Overwrites the content of the file based on the current commit.")
                .arg(
                    Arg::with_name("glob")
                        .long("glob")
                        .help("revert all the local changes that match the specified pattern"))
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
            SubCommand::with_name("resolves-pending")
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
        .subcommand(
            SubCommand::with_name("import-git-branch")
                .about("Replicates branches and commits from a git repo")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("Path to the root of a git repository. Should contain a .git subfolder"))
                .arg(
                    Arg::with_name("branch")
                        .required(true)
                        .help("Name of the branch to import"))
        )
        .subcommand(
            SubCommand::with_name("ping")
                .about("Contact server")
                .arg(
                    Arg::with_name("server_uri")
                        .required(true)
                        .help("lsc://host:port"))
        )
        .get_matches();

    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

    match matches.subcommand() {
        ("init-local-repository", Some(command_match)) => {
            log_str(LogLevel::Info, "init-local-repository");
            let path = command_match.value_of("directory").unwrap();
            if let Err(e) = tokio_runtime.block_on(
                legion_source_control::init_local_repository_command(Path::new(&path)),
            ) {
                return Err(e);
            }
            Ok(())
        }
        ("init-remote-repository", Some(command_match)) => {
            log_str(LogLevel::Info, "init-remote-repository");
            let repo_uri = command_match.value_of("uri").unwrap();
            let blob_uri = command_match.value_of("blob-storage");
            tokio_runtime.block_on(legion_source_control::init_remote_repository_command(
                repo_uri, blob_uri,
            ))
        }
        ("destroy-repository", Some(command_match)) => {
            log_str(LogLevel::Info, "destroy-repository");
            let repo_uri = command_match.value_of("uri").unwrap();
            tokio_runtime.block_on(
                legion_source_control::destroy_repository::destroy_repository_command(repo_uri),
            )
        }
        ("init-workspace", Some(command_match)) => {
            log_str(LogLevel::Info, "init-workspace");
            tokio_runtime.block_on(init_workspace_command(
                Path::new(command_match.value_of("workspace-directory").unwrap()),
                command_match.value_of("repository-uri").unwrap(),
            ))
        }
        ("add", Some(command_match)) => {
            let path_arg = command_match.value_of("path").unwrap();
            log_string(LogLevel::Info, format!("add {}", path_arg));
            tokio_runtime.block_on(track_new_file_command(Path::new(path_arg)))
        }
        ("edit", Some(command_match)) => {
            log_str(LogLevel::Info, "edit");
            tokio_runtime.block_on(edit_file_command(Path::new(
                command_match.value_of("path").unwrap(),
            )))
        }
        ("delete", Some(command_match)) => {
            log_str(LogLevel::Info, "delete");
            tokio_runtime.block_on(delete_file_command(Path::new(
                command_match.value_of("path").unwrap(),
            )))
        }
        ("lock", Some(command_match)) => {
            log_str(LogLevel::Info, "lock");
            tokio_runtime.block_on(lock_file_command(Path::new(
                command_match.value_of("path").unwrap(),
            )))
        }
        ("unlock", Some(command_match)) => {
            log_str(LogLevel::Info, "unlock");
            tokio_runtime.block_on(unlock_file_command(Path::new(
                command_match.value_of("path").unwrap(),
            )))
        }
        ("list-locks", Some(_command_match)) => {
            log_str(LogLevel::Info, "list-locks");
            tokio_runtime.block_on(list_locks_command())
        }
        ("diff", Some(command_match)) => {
            log_str(LogLevel::Info, "diff");
            let notool = command_match.is_present("notool");
            let reference_version_name = command_match.value_of("reference").unwrap_or("base");
            tokio_runtime.block_on(diff_file_command(
                Path::new(command_match.value_of("path").unwrap()),
                reference_version_name,
                !notool,
            ))
        }
        ("resolve", Some(command_match)) => {
            log_str(LogLevel::Info, "resolve");
            let notool = command_match.is_present("notool");
            let path = Path::new(command_match.value_of("path").unwrap());
            tokio_runtime.block_on(resolve_file_command(path, !notool))
        }
        ("create-branch", Some(command_match)) => {
            log_str(LogLevel::Info, "create-branch");
            let name = command_match.value_of("name").unwrap();
            tokio_runtime.block_on(create_branch_command(name))
        }
        ("merge-branch", Some(command_match)) => {
            log_str(LogLevel::Info, "merge-branch");
            let name = command_match.value_of("name").unwrap();
            merge_branch_command(&tokio_runtime, name)
        }
        ("switch-branch", Some(command_match)) => {
            log_str(LogLevel::Info, "switch-branch");
            let name = command_match.value_of("name").unwrap();
            switch_branch_command(&tokio_runtime, name)
        }
        ("detach-branch", Some(_command_match)) => {
            log_str(LogLevel::Info, "detach-branch");
            tokio_runtime.block_on(detach_branch_command())
        }
        ("attach-branch", Some(command_match)) => {
            let parent_branch_name = command_match.value_of("parent-branch-name").unwrap();
            log_string(
                LogLevel::Info,
                format!("attach-branch {}", parent_branch_name),
            );
            tokio_runtime.block_on(attach_branch_command(parent_branch_name))
        }
        ("list-branches", Some(_command_match)) => {
            log_str(LogLevel::Info, "list-branches");
            tokio_runtime.block_on(list_branches_command())
        }
        ("revert", Some(command_match)) => {
            let path = command_match.value_of("path").unwrap();
            log_string(LogLevel::Info, format!("revert {}", path));
            if command_match.is_present("glob") {
                tokio_runtime.block_on(revert_glob_command(path))
            } else {
                tokio_runtime.block_on(revert_file_command(Path::new(path)))
            }
        }
        ("commit", Some(command_match)) => {
            let mut message = String::from("");
            for item in command_match.values_of("message").unwrap() {
                message += item;
            }
            log_string(LogLevel::Info, format!("commit {:?}", message));
            tokio_runtime.block_on(commit_command(&message))
        }
        ("local-changes", Some(_command_match)) => {
            log_str(LogLevel::Info, "local-changes");
            match tokio_runtime.block_on(find_local_changes_command()) {
                Ok(changes) => {
                    if changes.is_empty() {
                        println!("No local changes");
                    }
                    for change in changes {
                        println!("{:?} {}", change.change_type, change.relative_path);
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        ("resolves-pending", Some(_command_match)) => {
            log_str(LogLevel::Info, "resolves-pending");
            match tokio_runtime.block_on(find_resolves_pending_command()) {
                Ok(resolves_pending) => {
                    if resolves_pending.is_empty() {
                        println!("No local changes need to be resolved");
                    }
                    for m in resolves_pending {
                        println!(
                            "{} {} {}",
                            m.relative_path, &m.base_commit_id, &m.theirs_commit_id
                        );
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        ("sync", Some(command_match)) => {
            log_str(LogLevel::Info, "sync");
            match command_match.value_of("commit-id") {
                Some(commit_id) => tokio_runtime.block_on(sync_to_command(commit_id)),
                None => tokio_runtime.block_on(sync_command()),
            }
        }
        ("log", Some(_command_match)) => {
            log_str(LogLevel::Info, "log");
            tokio_runtime.block_on(log_command())
        }
        ("config", Some(_command_match)) => {
            log_str(LogLevel::Info, "config");
            print_config_command()
        }
        ("import-git-branch", Some(command_match)) => {
            let path_arg = command_match.value_of("path").unwrap();
            let branch_name = command_match.value_of("branch").unwrap();
            log_string(
                LogLevel::Info,
                format!("import-git-branch {} {} ", path_arg, branch_name),
            );
            import_git_branch_command(Path::new(path_arg), branch_name)
        }
        ("ping", Some(command_match)) => {
            let server_uri = command_match.value_of("server_uri").unwrap();
            log_string(LogLevel::Info, format!("ping {}", server_uri));
            tokio_runtime.block_on(ping_console_command(server_uri))
        }
        other_match => {
            log_str(LogLevel::Info, "unknown subcommand match");
            Err(format!("unknown subcommand match: {:?}", &other_match))
        }
    }
}
