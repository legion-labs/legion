// BEGIN - Legion Labs lints v0.2
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::string_add_assign,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::trait_duplication_in_bounds,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::useless_transmute,
    clippy::use_self,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    future_incompatible,
    nonstandard_style,
    broken_intra_doc_links,
    private_intra_doc_links,
    missing_crate_level_docs,
    rust_2018_idioms
)]
// END - Legion Labs standard lints v0.2
// crate-specific exceptions:
#![allow()]

use clap::{App, AppSettings, Arg, SubCommand};
use legion_src_ctl::*;
use std::path::Path;

fn main() {
    if let Err(e) = main_impl() {
        println!("{}", e);
        std::process::exit(1);
    }
}

fn main_impl() -> Result<(), String> {
    let matches = App::new("Legion Source Control")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .about("CLI to interact with Legion Source Control")
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
            SubCommand::with_name("import-git-repo")
                .about("Replicates branches and commits from a git repo")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("Path to the root of a git repository. Should contain a .git subfolder"))
                .arg(
                    Arg::with_name("branch")
                        .help("Name of the branch to import. If omitted will default to first valid branch found"))
        )
        .get_matches();

    match matches.subcommand() {
        ("init-local-repository", Some(command_match)) => legion_src_ctl::init_local_repository(
            Path::new(command_match.value_of("repository-directory").unwrap()),
        ),
        ("init-workspace", Some(command_match)) => init_workspace(
            Path::new(command_match.value_of("workspace-directory").unwrap()),
            Path::new(command_match.value_of("repository-directory").unwrap()),
        ),
        ("add", Some(command_match)) => {
            track_new_file_command(Path::new(command_match.value_of("path").unwrap()))
        }
        ("edit", Some(command_match)) => {
            edit_file_command(Path::new(command_match.value_of("path").unwrap()))
        }
        ("delete", Some(command_match)) => {
            delete_file_command(Path::new(command_match.value_of("path").unwrap()))
        }
        ("lock", Some(command_match)) => {
            lock_file_command(Path::new(command_match.value_of("path").unwrap()))
        }
        ("unlock", Some(command_match)) => {
            unlock_file_command(Path::new(command_match.value_of("path").unwrap()))
        }
        ("list-locks", Some(_command_match)) => list_locks_command(),
        ("diff", Some(command_match)) => {
            let notool = command_match.is_present("notool");
            let reference_version_name = command_match.value_of("reference").unwrap_or("base");
            diff_file_command(
                Path::new(command_match.value_of("path").unwrap()),
                reference_version_name,
                !notool,
            )
        }
        ("resolve", Some(command_match)) => {
            let notool = command_match.is_present("notool");
            let path = Path::new(command_match.value_of("path").unwrap());
            resolve_file_command(path, !notool)
        }
        ("create-branch", Some(command_match)) => {
            let name = command_match.value_of("name").unwrap();
            create_branch_command(name)
        }
        ("merge-branch", Some(command_match)) => {
            let name = command_match.value_of("name").unwrap();
            merge_branch_command(name)
        }
        ("switch-branch", Some(command_match)) => {
            let name = command_match.value_of("name").unwrap();
            switch_branch_command(name)
        }
        ("detach-branch", Some(_command_match)) => detach_branch_command(),
        ("attach-branch", Some(command_match)) => {
            let name = command_match.value_of("parent-branch-name").unwrap();
            attach_branch_command(name)
        }
        ("list-branches", Some(_command_match)) => list_branches_command(),
        ("revert", Some(command_match)) => {
            let path = command_match.value_of("path").unwrap();
            if command_match.is_present("glob") {
                revert_glob_command(path)
            } else {
                revert_file_command(Path::new(path))
            }
        }
        ("commit", Some(command_match)) => {
            let mut message = String::from("");
            for item in command_match.values_of("message").unwrap() {
                message += item;
            }
            commit_command(&message)
        }
        ("local-changes", Some(_command_match)) => match find_local_changes_command() {
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
        },
        ("resolves-pending", Some(_command_match)) => match find_resolves_pending_command() {
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
        },
        ("sync", Some(command_match)) => match command_match.value_of("commit-id") {
            Some(commit_id) => sync_to_command(commit_id),
            None => sync_command(),
        },
        ("log", Some(_command_match)) => log_command(),
        ("config", Some(_command_match)) => print_config_command(),
        ("import-git-repo", Some(command_match)) => import_git_repo_command(
            Path::new(command_match.value_of("path").unwrap()),
            command_match.value_of("branch"),
        ),
        other_match => Err(format!("unknown subcommand match: {:?}", &other_match)),
    }
}
