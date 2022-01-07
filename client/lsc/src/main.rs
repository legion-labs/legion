//! Source control CLI

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

use std::{path::Path};

use clap::{AppSettings, Parser, Subcommand};
use lgn_source_control::*;
use lgn_telemetry::*;
use lgn_telemetry_sink::TelemetryGuard;

/// Legion Source Control
#[derive(Parser, Debug)]
#[clap(name = "Legion Source Control")]
#[clap(about = "CLI to interact with Legion Source Control", version, author)]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initializes a repository stored on a local or remote system
    #[clap(name = "create-repository")]
    CreateRepository {
        /// The repository URL.
        repository_url: RepositoryUrl,
        // The optional blob storage URL. If none is specified, one will be
        // guessed from the repository URL.
        blob_storage_url: Option<BlobStorageUrl>,
    },
    /// Destroys all repository data permanently
    #[clap(name = "destroy-repository")]
    DestroyRepository {
        /// The repository URL.
        repository_url: RepositoryUrl,
    },
    /// Initializes a workspace and populates it with the latest version of the main branch
    #[clap(name = "init-workspace")]
    InitWorkspace {
        /// lsc workspace directory
        workspace_directory: String,
        /// uri printed at the creation of the repository
        repository_uri: String,
    },
    /// Adds local file to the set of pending changes
    #[clap(name = "add")]
    Add {
        /// local path within a workspace
        path: String,
    },
    /// Makes file writable and adds it to the set of pending changes
    #[clap(name = "edit")]
    Edit {
        /// local path within a workspace
        path: String,
    },
    /// Deletes the local file and records the pending change
    #[clap(name = "delete")]
    Delete {
        /// local path within a workspace
        path: String,
    },
    /// Prevent others from modifying the specified file. Locks apply throught all related branches
    #[clap(name = "lock")]
    Lock {
        /// local path within a workspace
        path: String,
    },
    /// Releases a lock, allowing others to modify or lock the file
    #[clap(name = "unlock")]
    Unlock {
        /// local path within a workspace
        path: String,
    },
    /// Prints all the locks in the current lock domain
    #[clap(name = "list-locks")]
    ListLocks,
    /// Prints difference between local file and specified commit
    #[clap(name = "diff")]
    Diff {
        /// ignores diff tool config and prints a patch on stdout
        #[clap(long)]
        notool: bool,
        /// local path within a workspace
        path: String,
        /// reference version: a commit id, base or latest
        #[clap(default_value = "base")]
        reference: String,
    },
    /// Reconciles local modifications with colliding changes from other workspaces
    #[clap(name = "resolve")]
    Resolve {
        /// ignores diff tool config and prints a patch on stdout
        #[clap(long)]
        notool: bool,
        /// local path within a workspace
        path: String,
    },
    /// Creates a new branch based on the state of the workspace
    #[clap(name = "create-branch")]
    CreateBranch {
        /// name of the new branch
        name: String,
    },
    /// Merge the specified branch into the current one
    #[clap(name = "merge-branch")]
    MergeBranch {
        /// name of the branch to merge
        name: String,
    },
    /// Syncs workspace to specified branch
    #[clap(name = "switch-branch")]
    SwitchBranch {
        /// name of the existing branch to sync to
        name: String,
    },
    /// Move the current branch and its descendance to a new lock domain
    #[clap(name = "detach-branch")]
    DetachBranch,
    /// Merges the lock domains of the two branches
    #[clap(name = "attach-branch")]
    AttachBranches {
        /// name of the existing branch to set as parent
        parent_branch_name: String,
    },
    /// Prints a list of all branches
    #[clap(name = "list-branches")]
    ListBranches,
    /// Abandon the local changes made to a file. Overwrites the content of the file based on the current commit.
    #[clap(name = "revert")]
    Revert {
        /// revert all the local changes that match the specified pattern
        #[clap(long)]
        glob: bool,
        /// local path within a workspace
        path: String,
    },
    /// Lists changes in workspace lsc knows about
    #[clap(name = "local-changes")]
    LocalChanges,
    /// Lists commits of the current branch
    #[clap(name = "log")]
    Log,
    /// Updates the workspace with the latest version of the files
    #[clap(name = "sync")]
    Sync {
        /// version to sync to
        commit_id: Option<String>,
    },
    /// Lists the files that are scheduled to be merged following a sync with colliding changes
    #[clap(name = "resolves-pending")]
    ResolvesPending,
    /// Records local changes in the repository as a single transaction
    #[clap(name = "commit")]
    Commit {
        /// commit message
        #[clap(short, value_delimiter = '\"')]
        message: Vec<String>,
    },
    /// Prints the path to the configuration file and its content
    #[clap(name = "config")]
    Config,
    /// Replicates branches and commits from a git repo
    #[clap(name = "import-git-branch")]
    ImportGitBranch {
        /// Path to the root of a git repository. Should contain a .git subfolder
        path: String,
        /// Name of the branch to import
        branch: String,
    },
    /// Contact server
    #[clap(name = "ping")]
    Ping {
        /// The repository URL.
        repository_url: RepositoryUrl,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _telemetry_guard = TelemetryGuard::new().unwrap();

            //let repository_url = command_match
            //    .value_of(ARG_REPOSITORY_URL)
            //    .map(RepositoryUrl::from_str)
            //    .transpose()?
            //    .unwrap_or_else(RepositoryUrl::from_current_dir)
            //    .make_absolute(std::env::current_dir()?);

            //let blob_storage_url = command_match
            //    .value_of(ARG_BLOB_STORAGE_URL)
            //    .map(BlobStorageUrl::from_str)
            //    .transpose()?
            //    .map(|url| std::env::current_dir().map(|d| url.make_absolute(d)))
            //    .transpose()?;

    let args = Cli::parse();

    match args.command {
        Commands::CreateRepository { repository_url, blob_storage_url } => {
            info!("create-repository");

            lgn_source_control::commands::create_repository(&repository_url, &blob_storage_url)
                .await?;

            Ok(())
        }
        Commands::DestroyRepository { repository_url } => {
            info!("destroy-repository");

            lgn_source_control::commands::destroy_repository(&repository_url).await?;

            Ok(())
        }
        Commands::InitWorkspace {
            workspace_directory,
            repository_uri,
        } => {
            info!("init-workspace");

            init_workspace_command(Path::new(&workspace_directory), &repository_uri).await
        }
        Commands::Add { path } => {
            info!("add {}", path);
            track_new_file_command(Path::new(&path)).await
        }
        Commands::Edit { path } => {
            info!("edit");
            edit_file_command(Path::new(&path)).await
        }
        Commands::Delete { path } => {
            info!("delete");
            delete_file_command(Path::new(&path)).await
        }
        Commands::Lock { path } => {
            info!("lock");
            lock_file_command(Path::new(&path)).await
        }
        Commands::Unlock { path } => {
            info!("unlock");
            unlock_file_command(Path::new(&path)).await
        }
        Commands::ListLocks => {
            info!("list-locks");
            list_locks_command().await
        }
        Commands::Diff {
            notool,
            path,
            reference,
        } => {
            info!("diff");
            diff_file_command(Path::new(&path), &reference, !notool).await
        }
        Commands::Resolve { notool, path } => {
            info!("resolve");
            resolve_file_command(Path::new(&path), !notool).await
        }
        Commands::CreateBranch { name } => {
            info!("create-branch");
            create_branch_command(&name).await
        }
        Commands::MergeBranch { name } => {
            info!("merge-branch");
            merge_branch_command(&name).await
        }
        Commands::SwitchBranch { name } => {
            info!("switch-branch");
            switch_branch_command(&name).await
        }
        Commands::DetachBranch => {
            info!("detach-branch");
            detach_branch_command().await
        }
        Commands::AttachBranches { parent_branch_name } => {
            info!("attach-branch {}", parent_branch_name);
            attach_branch_command(&parent_branch_name).await
        }
        Commands::ListBranches => {
            info!("list-branches");
            list_branches_command().await
        }
        Commands::Revert { glob, path } => {
            info!("revert {}", path);
            if glob {
                revert_glob_command(&path).await
            } else {
                revert_file_command(Path::new(&path)).await
            }
        }
        Commands::LocalChanges => {
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
        Commands::Log => {
            info!("log");

            log_command().await
        }
        Commands::Sync { commit_id } => {
            info!("sync");
            match commit_id {
                Some(commit_id) => sync_to_command(&commit_id).await,
                None => sync_command().await,
            }
        }
        Commands::ResolvesPending => {
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
        Commands::Commit { message } => {
            let mut aggregate_message = String::from("");
            for item in message {
                aggregate_message += &item;
            }
            info!("commit {:?}", aggregate_message);

            commit_command(&aggregate_message).await
        }
        Commands::Config => {
            info!("config");

            print_config_command()
        }
        Commands::ImportGitBranch { path, branch } => {
            info!("import-git-branch {} {} ", path, branch);
            import_git_branch_command(Path::new(&path), &branch).await
        }
        Commands::Ping { repository_url } => {
            info!("ping {}", &repository_url);

            lgn_source_control::commands::ping(&repository_url).await
        }
    }
}
