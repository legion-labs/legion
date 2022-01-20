//! Source control CLI

// crate-specific lint exceptions:
#![allow(clippy::exit, clippy::wildcard_imports)]

use std::path::PathBuf;

use clap::{AppSettings, Parser, Subcommand};
use lgn_source_control::*;
use lgn_telemetry_sink::{Config, TelemetryGuard};
use lgn_tracing::*;

/// Legion Source Control
#[derive(Parser, Debug)]
#[clap(name = "Legion Source Control")]
#[clap(about = "CLI to interact with Legion Source Control", version, author)]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    #[clap(name = "debug", short, long, help = "Enable debug logging")]
    debug: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initializes an index stored on a local or remote system
    #[clap(name = "create-index")]
    CreateIndex {
        /// The index URL.
        index_url: String,
    },
    /// Destroys all index data permanently
    #[clap(name = "destroy-index")]
    DestroyIndex {
        /// The index URL.
        index_url: String,
    },
    /// Checks if an index exists.
    #[clap(name = "index-exists")]
    IndexExists {
        /// The index URL.
        index_url: String,
    },
    /// Initializes a workspace and populates it with the latest version of the main branch
    #[clap(name = "init-workspace")]
    InitWorkspace {
        /// lsc workspace directory
        workspace_directory: PathBuf,
        /// uri printed at the creation of the repository
        index_url: String,
    },
    /// Adds local file to the set of pending changes
    #[clap(name = "add")]
    Add {
        /// local path within a workspace
        path: PathBuf,
    },
    /// Makes file writable and adds it to the set of pending changes
    #[clap(name = "edit")]
    Edit {
        /// local path within a workspace
        path: PathBuf,
    },
    /// Deletes the local file and records the pending change
    #[clap(name = "delete")]
    Delete {
        /// local path within a workspace
        path: PathBuf,
    },
    /// Prevent others from modifying the specified file. Locks apply throught all related branches
    #[clap(name = "lock")]
    Lock {
        /// local path within a workspace
        path: PathBuf,
    },
    /// Releases a lock, allowing others to modify or lock the file
    #[clap(name = "unlock")]
    Unlock {
        /// local path within a workspace
        path: PathBuf,
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
        path: PathBuf,
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
        path: PathBuf,
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
        path: PathBuf,
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
        path: PathBuf,
        /// Name of the branch to import
        branch: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let _telemetry_guard = if args.debug {
        TelemetryGuard::default()
            .unwrap()
            .with_log_level(LevelFilter::Debug)
    } else {
        TelemetryGuard::new(Config::default(), false)
            .unwrap()
            .with_log_level(LevelFilter::Info)
    };

    span_scope!("lsc::main");

    match args.command {
        Commands::CreateIndex { index_url } => {
            println!("Creating index at: {}", &index_url);

            let index = Index::new(&index_url)?;

            index
                .create()
                .await
                .map_err::<anyhow::Error, _>(Into::into)?;

            Ok(())
        }
        Commands::DestroyIndex { index_url } => {
            println!("Destroying index at: {}", &index_url);

            let index = Index::new(&index_url)?;

            index.destroy().await.map_err(Into::into)
        }
        Commands::IndexExists { index_url } => {
            let index = Index::new(&index_url)?;

            if index
                .exists()
                .await
                .map_err::<anyhow::Error, _>(Into::into)?
            {
                println!("The index exists");
            } else {
                println!("The index does not exist");
            }

            Ok(())
        }
        Commands::InitWorkspace {
            workspace_directory,
            index_url,
        } => {
            info!("init-workspace");

            let config = WorkspaceConfig {
                index_url: index_url.clone(),
                registration: WorkspaceRegistration::new_with_current_user(),
            };

            Workspace::init(&workspace_directory, config)
                .await
                .map_err(Into::into)
                .map(|_| ())
        }
        Commands::Add { path } => track_new_file_command(path).await,
        Commands::Edit { path } => edit_file_command(path).await,
        Commands::Delete { path } => {
            info!("delete");
            delete_file_command(path).await
        }
        Commands::Lock { path } => {
            info!("lock");
            lock_file_command(path).await
        }
        Commands::Unlock { path } => {
            info!("unlock");
            unlock_file_command(path).await
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
            diff_file_command(path, &reference, !notool).await
        }
        Commands::Resolve { notool, path } => {
            info!("resolve");
            resolve_file_command(path, !notool).await
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
            if glob {
                revert_glob_command(path.to_str().unwrap()).await
            } else {
                revert_file_command(path).await
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
            import_git_branch_command(path, &branch).await
        }
    }
}
