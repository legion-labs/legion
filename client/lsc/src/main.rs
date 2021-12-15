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

use std::{path::Path, str::FromStr};

use ::log::info;
use clap::{App, AppSettings, Arg, SubCommand};

use lgn_source_control::*;
use lgn_telemetry::*;

const SUB_COMMAND_CREATE_REPOSITORY: &str = "create-repository";
const SUB_COMMAND_CREATE_REMOTE_REPOSITORY: &str = "create-remote-repository";

const ARG_REPOSITORY_URL: &str = "repository-url";
const ARG_BLOB_STORAGE_URL: &str = "blob-storage-url";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    lgn_logger::Logger::init(lgn_logger::Config::default()).unwrap();
    let _telemetry_guard = TelemetrySystemGuard::new();
    let _telemetry_thread_guard = TelemetryThreadGuard::new();

    trace_scope!();

    let matches = App::new("Legion Source Control")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .about("CLI to interact with Legion Source Control")
        .subcommand(
            SubCommand::with_name(SUB_COMMAND_CREATE_REPOSITORY)
                .about("Create a new local repository that uses the filesystem as its storage backend")
                .arg(
                    Arg::with_name(ARG_REPOSITORY_URL)
                        .help("The local path to the repository. If not specified, uses the current directory")
                )
                .arg(
                    Arg::with_name(ARG_BLOB_STORAGE_URL)
                        .help("The blob storage URL. If not specified and no default blob storage can be determined, an error will be reported. Example: file://somepath, s3://bucket/root")
                )
        )
        .subcommand(
            SubCommand::with_name(SUB_COMMAND_CREATE_REMOTE_REPOSITORY)
                .about("Create a repository on a remote server or database")
                .arg(
                    Arg::with_name(ARG_REPOSITORY_URL)
                        .required(true)
                        .help("The remote repository URL. Example: mysql://user:pass@host:port/database, lsc://host:port/database")
                )
                .arg(
                    Arg::with_name(ARG_BLOB_STORAGE_URL)
                        .help("The blob storage URL. If not specified and no default blob storage can be determined, an error will be reported. Example: file://somepath, s3://bucket/root")
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

    match matches.subcommand() {
        (SUB_COMMAND_CREATE_REPOSITORY, Some(command_match)) => {
            let repository_url = command_match
                .value_of(ARG_REPOSITORY_URL)
                .map(RepositoryUrl::from_str)
                .transpose()?
                .unwrap_or_else(RepositoryUrl::from_current_dir)
                .make_absolute(std::env::current_dir()?);

            let blob_storage_url = command_match
                .value_of(ARG_BLOB_STORAGE_URL)
                .map(BlobStorageUrl::from_str)
                .transpose()?
                .map(|url| std::env::current_dir().map(|d| url.make_absolute(d)))
                .transpose()?;

            lgn_source_control::commands::create_repository(&repository_url, &blob_storage_url)
                .await
        }
        (SUB_COMMAND_CREATE_REMOTE_REPOSITORY, Some(command_match)) => {
            let repo_uri = command_match.value_of(ARG_REPOSITORY_URL).unwrap();
            let blob_uri = command_match.value_of(ARG_BLOB_STORAGE_URL);

            lgn_source_control::commands::create_remote_repository_command(repo_uri, blob_uri).await
        }
        ("destroy-repository", Some(command_match)) => {
            info!("destroy-repository");
            let repo_uri = command_match.value_of("uri").unwrap();

            lgn_source_control::destroy_repository::destroy_repository_command(repo_uri).await
        }
        ("init-workspace", Some(command_match)) => {
            info!("init-workspace");

            init_workspace_command(
                Path::new(command_match.value_of("workspace-directory").unwrap()),
                command_match.value_of("repository-uri").unwrap(),
            )
            .await
        }
        ("add", Some(command_match)) => {
            let path_arg = command_match.value_of("path").unwrap();
            info!("add {}", path_arg);

            track_new_file_command(Path::new(path_arg)).await
        }
        ("edit", Some(command_match)) => {
            info!("edit");

            edit_file_command(Path::new(command_match.value_of("path").unwrap())).await
        }
        ("delete", Some(command_match)) => {
            info!("delete");

            delete_file_command(Path::new(command_match.value_of("path").unwrap())).await
        }
        ("lock", Some(command_match)) => {
            info!("lock");

            lock_file_command(Path::new(command_match.value_of("path").unwrap())).await
        }
        ("unlock", Some(command_match)) => {
            info!("unlock");

            unlock_file_command(Path::new(command_match.value_of("path").unwrap())).await
        }
        ("list-locks", Some(_command_match)) => {
            info!("list-locks");

            list_locks_command().await
        }
        ("diff", Some(command_match)) => {
            info!("diff");
            let notool = command_match.is_present("notool");
            let reference_version_name = command_match.value_of("reference").unwrap_or("base");

            diff_file_command(
                Path::new(command_match.value_of("path").unwrap()),
                reference_version_name,
                !notool,
            )
            .await
        }
        ("resolve", Some(command_match)) => {
            info!("resolve");
            let notool = command_match.is_present("notool");
            let path = Path::new(command_match.value_of("path").unwrap());

            resolve_file_command(path, !notool).await
        }
        ("create-branch", Some(command_match)) => {
            info!("create-branch");
            let name = command_match.value_of("name").unwrap();

            create_branch_command(name).await
        }
        ("merge-branch", Some(command_match)) => {
            info!("merge-branch");
            let name = command_match.value_of("name").unwrap();

            merge_branch_command(name).await
        }
        ("switch-branch", Some(command_match)) => {
            info!("switch-branch");
            let name = command_match.value_of("name").unwrap();

            switch_branch_command(name).await
        }
        ("detach-branch", Some(_command_match)) => {
            info!("detach-branch");

            detach_branch_command().await
        }
        ("attach-branch", Some(command_match)) => {
            let parent_branch_name = command_match.value_of("parent-branch-name").unwrap();
            info!("attach-branch {}", parent_branch_name);

            attach_branch_command(parent_branch_name).await
        }
        ("list-branches", Some(_command_match)) => {
            info!("list-branches");

            list_branches_command().await
        }
        ("revert", Some(command_match)) => {
            let path = command_match.value_of("path").unwrap();
            info!("revert {}", path);

            if command_match.is_present("glob") {
                revert_glob_command(path).await
            } else {
                revert_file_command(Path::new(path)).await
            }
        }
        ("commit", Some(command_match)) => {
            let mut message = String::from("");
            for item in command_match.values_of("message").unwrap() {
                message += item;
            }
            info!("commit {:?}", message);

            commit_command(&message).await
        }
        ("local-changes", Some(_command_match)) => {
            info!("local-changes");

            find_local_changes_command().await.map(|changes| {
                if changes.is_empty() {
                    println!("No local changes");
                }

                for change in changes {
                    println!("{:?} {}", change.change_type, change.relative_path);
                }
            })
        }
        ("resolves-pending", Some(_command_match)) => {
            info!("resolves-pending");

            find_resolves_pending_command()
                .await
                .map(|resolves_pending| {
                    if resolves_pending.is_empty() {
                        println!("No local changes need to be resolved");
                    }

                    for m in resolves_pending {
                        println!(
                            "{} {} {}",
                            m.relative_path, &m.base_commit_id, &m.theirs_commit_id
                        );
                    }
                })
        }
        ("sync", Some(command_match)) => {
            info!("sync");

            match command_match.value_of("commit-id") {
                Some(commit_id) => sync_to_command(commit_id).await,
                None => sync_command().await,
            }
        }
        ("log", Some(_command_match)) => {
            info!("log");

            log_command().await
        }
        ("config", Some(_command_match)) => {
            info!("config");

            print_config_command()
        }
        ("import-git-branch", Some(command_match)) => {
            let path_arg = command_match.value_of("path").unwrap();
            let branch_name = command_match.value_of("branch").unwrap();
            info!("import-git-branch {} {} ", path_arg, branch_name);

            import_git_branch_command(Path::new(path_arg), branch_name).await
        }
        ("ping", Some(command_match)) => {
            let server_uri = command_match.value_of("server_uri").unwrap();
            info!("ping {}", server_uri);

            ping_console_command(server_uri).await
        }
        other_match => {
            info!("unknown subcommand match");

            anyhow::bail!("unknown subcommand match: {:?}", &other_match)
        }
    }
}
