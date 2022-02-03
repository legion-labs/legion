//! Source control CLI

// crate-specific lint exceptions:
#![allow(clippy::exit, clippy::wildcard_imports)]

use std::path::PathBuf;

use clap::{AppSettings, Parser, Subcommand};
use lgn_source_control::*;
use lgn_telemetry_sink::{Config, TelemetryGuard};
use lgn_tracing::*;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

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

    #[clap(name = "no-color", long, help = "Disable color output")]
    no_color: bool,
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
    #[clap(name = "init-workspace", alias = "init")]
    InitWorkspace {
        /// lsc workspace directory
        workspace_directory: PathBuf,
        /// uri printed at the creation of the repository
        index_url: String,
    },
    /// Adds local file to the set of pending changes
    #[clap(name = "add")]
    Add {
        /// A list of paths for files to add.
        #[clap(required = true, parse(from_os_str))]
        paths: Vec<PathBuf>,
    },
    /// Makes file writable and adds it to the set of pending changes
    #[clap(name = "edit")]
    Edit {
        /// A list of paths for files to edit.
        #[clap(required = true, parse(from_os_str))]
        paths: Vec<PathBuf>,
    },
    /// Deletes the local file and records the pending change
    #[clap(name = "delete")]
    Delete {
        /// A list of paths for files to delete.
        #[clap(required = true, parse(from_os_str))]
        paths: Vec<PathBuf>,
    },
    /// Prevent others from modifying the specified file. Locks apply throught all related branches
    #[clap(name = "lock")]
    Lock {
        /// A list of paths for files to lock.
        #[clap(required = true, parse(from_os_str))]
        paths: Vec<PathBuf>,
    },
    /// Releases a lock, allowing others to modify or lock the file
    #[clap(name = "unlock")]
    Unlock {
        /// A list of paths for files to unlock.
        #[clap(required = true, parse(from_os_str))]
        paths: Vec<PathBuf>,
    },
    /// Prints all the locks in the current lock domain
    #[clap(name = "locks")]
    Locks,
    /// Prints difference between local file and specified commit
    #[clap(name = "diff")]
    Diff {
        /// ignores diff tool config and prints a patch on stdout
        #[clap(long)]
        no_tool: bool,

        #[clap(required = true, parse(from_os_str))]
        paths: Vec<PathBuf>,

        /// reference version: a commit id, base or latest
        #[clap(default_value = "base")]
        reference: String,
    },
    /// Reconciles local modifications with colliding changes from other workspaces
    #[clap(name = "resolve")]
    Resolve {
        /// ignores diff tool config and prints a patch on stdout
        #[clap(long)]
        no_tool: bool,

        #[clap(required = true, parse(from_os_str))]
        paths: Vec<PathBuf>,
    },
    /// Creates a new branch based on the state of the workspace
    #[clap(name = "create-branch")]
    CreateBranch {
        /// name of the new branch
        branch_name: String,
    },
    /// Merge the specified branch into the current one
    #[clap(name = "merge")]
    Merge {
        /// name of the branch to merge
        branch_name: String,
    },
    /// Checkout another branch.
    #[clap(name = "checkout")]
    Checkout {
        /// name of the existing branch to sync to
        branch_name: String,
    },
    /// Move the current branch and its descendance to a new lock domain
    #[clap(name = "detach")]
    Detach,
    /// Merges the lock domains of the two branches
    #[clap(name = "attach")]
    Attach {
        /// name of the existing branch to set as parent
        branch_name: String,
    },
    /// Prints a list of all branches
    #[clap(name = "branches")]
    Branches,
    /// Abandon the local changes made to a file. By default will revert both
    /// staged and unstaged changes.
    #[clap(name = "revert")]
    Revert {
        #[clap(required = true, parse(from_os_str))]
        paths: Vec<PathBuf>,

        /// Only revert staged changes. Will not modify files on disk. Changed edited files will remain in edit mode.
        #[clap(long, conflicts_with = "unstaged")]
        staged: bool,

        /// Only revert unstaged changes.
        #[clap(long, conflicts_with = "staged")]
        unstaged: bool,
    },
    /// Lists staged changes in workspace.
    #[clap(name = "status", alias = "st")]
    Status {
        /// Only list staged changes.
        #[clap(long, conflicts_with = "unstaged")]
        staged: bool,

        /// Only list unstaged changes.
        #[clap(long, conflicts_with = "staged")]
        unstaged: bool,
    },
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
    #[clap(name = "commit", alias = "ci")]
    Commit {
        /// commit message
        #[clap(short)]
        message: String,
    },
}

fn binary_name() -> String {
    "lsc".to_string()
}

fn green() -> ColorSpec {
    let mut colorspec = ColorSpec::new();

    colorspec.set_fg(Some(Color::Green)).set_intense(true);

    colorspec
}

fn yellow() -> ColorSpec {
    let mut colorspec = ColorSpec::new();

    colorspec.set_fg(Some(Color::Yellow)).set_intense(true);

    colorspec
}

fn red() -> ColorSpec {
    let mut colorspec = ColorSpec::new();

    colorspec.set_fg(Some(Color::Red));

    colorspec
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let _telemetry_guard = if args.debug {
        TelemetryGuard::default()
            .unwrap()
            .with_log_level(LevelFilter::Debug)
    } else {
        let mut config = Config::default();
        config.enable_console_printer = false;
        TelemetryGuard::new(config)
            .unwrap()
            .with_log_level(LevelFilter::Info)
    };

    span_scope!("lsc::main");
    let choice = match args.no_color {
        true => ColorChoice::Never,
        false => {
            if atty::is(atty::Stream::Stdout) {
                ColorChoice::Auto
            } else {
                ColorChoice::Never
            }
        }
    };

    let mut stdout = StandardStream::stdout(choice);

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

            let config =
                WorkspaceConfig::new(index_url, WorkspaceRegistration::new_with_current_user());

            Workspace::init(&workspace_directory, config)
                .await
                .map_err(Into::into)
                .map(|_| ())
        }
        Commands::Add { paths } => {
            let workspace = Workspace::find_in_current_directory().await?;

            workspace
                .add_files(paths.iter().map(PathBuf::as_path))
                .await
                .map_err(Into::into)
                .map(|_| ())
        }
        Commands::Edit { paths } => {
            let workspace = Workspace::find_in_current_directory().await?;

            workspace
                .edit_files(paths.iter().map(PathBuf::as_path))
                .await
                .map_err(Into::into)
                .map(|_| ())
        }
        Commands::Delete { paths } => {
            let workspace = Workspace::find_in_current_directory().await?;

            workspace
                .delete_files(paths.iter().map(PathBuf::as_path))
                .await
                .map_err(Into::into)
                .map(|_| ())
        }
        Commands::Lock { paths } => {
            info!("lock {:?}", paths);

            Ok(())
        }
        Commands::Unlock { paths } => {
            info!("unlock {:?}", paths);

            Ok(())
        }
        Commands::Locks => {
            info!("list-locks");

            Ok(())
        }
        Commands::Diff {
            no_tool: _,
            paths: _,
            reference: _,
        } => {
            info!("diff");

            Ok(())
        }
        Commands::Resolve {
            no_tool: _,
            paths: _,
        } => {
            info!("resolve");

            Ok(())
        }
        Commands::CreateBranch { branch_name } => {
            info!("create-branch: {}", branch_name);

            Ok(())
        }
        Commands::Merge { branch_name } => {
            info!("merge: {}", branch_name);

            Ok(())
        }
        Commands::Checkout { branch_name } => {
            info!("checkout: {}", branch_name);

            Ok(())
        }
        Commands::Detach => {
            info!("detach");

            Ok(())
        }
        Commands::Attach { branch_name } => {
            info!("attach {}", branch_name);

            Ok(())
        }
        Commands::Branches => {
            info!("list-branches");

            Ok(())
        }
        Commands::Revert {
            paths,
            staged,
            unstaged,
        } => {
            let workspace = Workspace::find_in_current_directory().await?;
            let staging = Staging::from_bool(staged, unstaged);

            let reverted_files = workspace
                .revert_files(paths.iter().map(PathBuf::as_path), staging)
                .await?;

            if reverted_files.is_empty() {
                println!("Nothing to revert");
            } else {
                println!("Reverted files:");

                let current_dir = std::env::current_dir()
                    .map_other_err("failed to determine current directory")?;

                for file in &reverted_files {
                    println!("   {}", workspace.make_relative_path(&current_dir, file));
                }
            }

            Ok(())
        }
        Commands::Status { staged, unstaged } => {
            let current_dir =
                std::env::current_dir().map_other_err("failed to determine current directory")?;
            let workspace = Workspace::find_in_current_directory().await?;
            let (branch, commit_id) = workspace.get_current_branch_and_commit().await?;
            let staging = Staging::from_bool(staged, unstaged);
            let (staged_changes, unstaged_changes) = workspace.status(staging).await?;

            println!("On branch {} (@{})", branch, commit_id);

            if !staged_changes.is_empty() {
                println!("\nChanges staged for commit:");

                for (path, change) in &staged_changes {
                    if change.change_type().has_modifications() {
                        stdout.set_color(&green())?;
                    } else {
                        stdout.set_color(&yellow())?;
                    }

                    print!(
                        "\t{:>8}:   {}",
                        change.change_type().to_human_string(),
                        workspace.make_relative_path(&current_dir, path),
                    );

                    if !change.change_type().has_modifications() {
                        stdout.reset()?;
                        print!(" (no modifications staged yet)");
                    }

                    println!("");
                }

                stdout.reset()?;
            }

            if !unstaged_changes.is_empty() {
                println!("\nChanges not staged for commit:");

                stdout.set_color(&red())?;

                for (path, change) in &unstaged_changes {
                    println!(
                        "\t{:>8}:   {}",
                        change.change_type().to_human_string(),
                        workspace.make_relative_path(&current_dir, path),
                    );
                }

                stdout.reset()?;
            }

            if staged_changes.is_empty() && unstaged_changes.is_empty() {
                println!("\nNo changes to commit");
            }

            Ok(())
        }
        Commands::Log => {
            info!("log");

            Ok(())
        }
        Commands::Sync { commit_id: _ } => {
            info!("sync");

            Ok(())
        }
        Commands::ResolvesPending => {
            info!("resolves-pending");

            Ok(())
        }
        Commands::Commit { message } => {
            let workspace = Workspace::find_in_current_directory().await?;

            match workspace.commit(&message).await {
                Ok(()) => Ok(()),
                Err(Error::UnchangedFilesMarkedForEdition { paths }) => {
                    let current_dir = std::env::current_dir()
                        .map_other_err("failed to determine current directory")?;

                    println!("The following files are marked for edition but do not have any change staged:");
                    println!(
                        "  (use \"{} add <file>...\" to update what will be commited)",
                        binary_name()
                    );
                    println!(
                        "  (use \"{} revert --staged <file>...\" to remove the edition mark)",
                        binary_name()
                    );

                    for path in &paths {
                        stdout.set_color(&red())?;
                        print!("\t{}", workspace.make_relative_path(&current_dir, path));
                        stdout.reset()?;
                        println!("");
                    }

                    println!("");

                    Err(anyhow::anyhow!("refusing to commit"))
                }
                Err(err) => Err(err.into()),
            }
        }
    }
}
