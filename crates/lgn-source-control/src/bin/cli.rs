//! Source control CLI

// crate-specific lint exceptions:
#![allow(clippy::exit, clippy::wildcard_imports)]

use std::{path::PathBuf, sync::Arc};

use clap::{Parser, Subcommand};
use lgn_source_control::*;
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::*;
// use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Legion Source Control
#[derive(Parser, Debug)]
#[clap(name = "Legion Source Control")]
#[clap(about = "CLI to interact with Legion Source Control", version, author)]
#[clap(arg_required_else_help(true))]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    #[clap(name = "debug", short, long, help = "Enable debug logging")]
    debug: bool,

    #[clap(name = "no-color", long, help = "Disable color output")]
    no_color: bool,

    #[clap(
        name = "lsc-dir",
        long,
        help = "Set the path to the repository (\".lsc\" directory)"
    )]
    lsc_dir: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initializes an index stored on a local or remote system
    #[clap(name = "create-repository")]
    CreateRepository {
        /// The index URL.
        repository_name: RepositoryName,
    },
    /// Destroys all index data permanently
    #[clap(name = "destroy-repository")]
    DestroyRepository {
        /// The index URL.
        repository_name: RepositoryName,
    },
    /// Checks if an index exists.
    #[clap(name = "repository-exists")]
    RepositoryExists {
        /// The index URL.
        repository_name: RepositoryName,
    },
    /// ListRepositories.
    #[clap(name = "list-repositories")]
    ListRepositories {},
    /// Initializes a workspace and populates it with the latest version of the main branch
    #[clap(name = "init-workspace", alias = "init")]
    InitWorkspace {
        /// uri printed at the creation of the repository
        repository_name: RepositoryName,
    },
    /// Adds local file to the set of pending changes
    #[clap(name = "add", alias = "a")]
    Add {
        /// A list of paths for files to add.
        #[clap(required = true, parse(from_os_str))]
        paths: Vec<PathBuf>,
    },
    /// Makes file writable and adds it to the set of pending changes
    #[clap(name = "checkout", alias = "co")]
    Checkout {
        /// A list of paths for files to edit.
        #[clap(required = true, parse(from_os_str))]
        paths: Vec<PathBuf>,
    },
    /// Deletes the local file and records the pending change
    #[clap(name = "delete", alias = "d")]
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
    /// Switch to another branch.
    #[clap(name = "switch")]
    Switch {
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
    #[clap(name = "branches", alias = "br")]
    Branches {
        /// Displays the full information about the branch
        #[clap(long)]
        full: bool,
    },
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
    Log {
        /// Display the log in short format.
        #[clap(long)]
        short: bool,
    },
    /// Updates the workspace with the latest version of the files
    #[clap(name = "sync")]
    Sync {
        /// Commit to sync to.
        commit_id: Option<CommitId>,
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

// fn binary_name() -> String {
//     "lsc".to_string()
// }

// fn green() -> ColorSpec {
//     let mut colorspec = ColorSpec::new();

//     colorspec.set_fg(Some(Color::Green)).set_intense(true);

//     colorspec
// }

// fn yellow() -> ColorSpec {
//     let mut colorspec = ColorSpec::new();

//     colorspec.set_fg(Some(Color::Yellow)).set_intense(true);

//     colorspec
// }

// fn red() -> ColorSpec {
//     let mut colorspec = ColorSpec::new();

//     colorspec.set_fg(Some(Color::Red));

//     colorspec
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    if let Some(lsc_dir) = args.lsc_dir {
        let lsc_dir = PathBuf::from(lsc_dir);
        let lsc_dir = if lsc_dir.is_absolute() {
            lsc_dir
        } else {
            std::env::current_dir().unwrap().join(lsc_dir)
        };

        std::env::set_current_dir(lsc_dir).unwrap();
    }

    let _telemetry_guard = if args.debug {
        TelemetryGuardBuilder::default()
            .with_local_sink_max_level(LevelFilter::Debug)
            .build()
    } else {
        TelemetryGuardBuilder::default()
            .with_local_sink_enabled(false)
            .build()
    };

    span_scope!("lsc::main");
    // let choice = if args.no_color {
    //     ColorChoice::Never
    // } else if atty::is(atty::Stream::Stdout) {
    //     ColorChoice::Auto
    // } else {
    //     ColorChoice::Never
    // };

    let _provider =
        Arc::new(lgn_content_store::Config::load_and_instantiate_persistent_provider().await?);
    let repository_index = Config::load_and_instantiate_repository_index().await?;

    //let mut stdout = StandardStream::stdout(choice);

    match args.command {
        Commands::CreateRepository { repository_name } => {
            println!("Creating repository: {}", &repository_name);

            repository_index.create_repository(&repository_name).await?;

            Ok(())
        }
        Commands::DestroyRepository { repository_name } => {
            println!("Destroying index at: {}", &repository_name);

            repository_index
                .destroy_repository(&repository_name)
                .await?;

            Ok(())
        }
        Commands::RepositoryExists { repository_name } => {
            match repository_index.load_repository(&repository_name).await {
                Ok(_) => {
                    println!("The repository exists");
                }
                Err(Error::RepositoryNotFound { repository_name }) => {
                    println!("The repository `{}` does not exist", repository_name);
                }
                Err(e) => {
                    return Err(e.into());
                }
            }

            Ok(())
        }
        Commands::ListRepositories {} => {
            match repository_index.list_repositories().await {
                Ok(repository_names) => {
                    if repository_names.is_empty() {
                        println!("No repositories found.");
                    } else {
                        println!("Listing {} repositorie(s):", repository_names.len());

                        for repository_name in repository_names {
                            println!("- {}", repository_name);
                        }
                    }
                }
                Err(e) => {
                    return Err(e.into());
                }
            }

            Ok(())
        }
        /*
        Commands::InitWorkspace { repository_name } => {
            info!("init-workspace");

            Workspace::init(repository_index, &repository_name, provider)
                .await
                .map_err(Into::into)
                .map(|_| ())
        }
        Commands::Add { paths } => {
            let workspace =
                Workspace::find_in_current_directory(repository_index, provider).await?;

            workspace
                .add_files(paths.iter().map(PathBuf::as_path))
                .await
                .map_err(Into::into)
                .map(|_| ())
        }
        Commands::Checkout { paths } => {
            let workspace =
                Workspace::find_in_current_directory(repository_index, provider).await?;

            workspace
                .checkout_files(paths.iter().map(PathBuf::as_path))
                .await
                .map_err(Into::into)
                .map(|_| ())
        }
        Commands::Delete { paths } => {
            let workspace =
                Workspace::find_in_current_directory(repository_index, provider).await?;

            workspace
                .delete_files(paths.iter().map(PathBuf::as_path))
                .await
                .map_err(Into::into)
                .map(|_| ())
        }
        */
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
        /*
        Commands::CreateBranch { branch_name } => {
            let workspace =
                Workspace::find_in_current_directory(repository_index, provider).await?;

            let branch = workspace.create_branch(&branch_name).await?;

            println!("Now on branch {}", branch);

            Ok(())
        }
        */
        Commands::Merge { branch_name } => {
            info!("merge: {}", branch_name);

            Ok(())
        }
        /*
        Commands::Switch { branch_name } => {
            let workspace =
                Workspace::find_in_current_directory(repository_index, provider).await?;

            let (branch, changes) = workspace.switch_branch(&branch_name).await?;

            println!("Now on branch {}", branch);

            print_changes(&workspace, &mut stdout, &changes)
        }
        */
        Commands::Detach => {
            info!("detach");

            Ok(())
        }
        Commands::Attach { branch_name } => {
            info!("attach {}", branch_name);

            Ok(())
        }
        /*
        Commands::Branches { full } => {
            let workspace =
                Workspace::find_in_current_directory(repository_index, provider).await?;

            let branches = workspace.get_branches().await?;

            if full {
                for branch in branches {
                    println!("{}", branch);
                }
            } else {
                for branch in branches {
                    println!("{}", branch.name);
                }
            }

            Ok(())
        }
        Commands::Revert {
            paths,
            staged,
            unstaged,
        } => {
            let workspace =
                Workspace::find_in_current_directory(repository_index, provider).await?;
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
            let workspace =
                Workspace::find_in_current_directory(repository_index, provider).await?;
            let current_branch = workspace.get_current_branch().await?;
            let staging = Staging::from_bool(staged, unstaged);
            let (staged_changes, unstaged_changes) = workspace.status(staging).await?;

            println!("On branch {}", current_branch);

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

                    println!();
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
        Commands::Log { short } => {
            let workspace =
                Workspace::find_in_current_directory(repository_index, provider).await?;
            let current_branch = workspace.get_current_branch().await?;
            let commits = workspace
                .list_commits(&ListCommitsQuery::single(current_branch.head))
                .await?;

            if short {
                for commit in &commits {
                    println!("{} {}", commit.id, commit.message);
                }
            } else {
                println!("Displaying commits for branch {}", current_branch);

                for commit in &commits {
                    stdout.set_color(&yellow())?;
                    println!("\ncommit {}", commit.id);
                    stdout.reset()?;
                    println!("Author: {}", commit.owner);
                    println!(
                        "Date:   {}",
                        chrono::DateTime::<chrono::Local>::from(commit.timestamp)
                            .format("%a %b %d %H:%M:%S %Y %z")
                    );
                    println!("\n    {}", commit.message);
                }
            }

            Ok(())
        }
        Commands::Sync { commit_id } => {
            let workspace =
                Workspace::find_in_current_directory(repository_index, provider).await?;

            let (current_commit_id, changes) = if let Some(commit_id) = commit_id {
                let changes = workspace.sync_to(commit_id).await?;

                (commit_id, changes)
            } else {
                let (branch, changes) = workspace.sync().await?;

                (branch.head, changes)
            };

            println!("Synced to {}", current_commit_id);

            print_changes(&workspace, &mut stdout, &changes)
        }
        */
        Commands::ResolvesPending => {
            info!("resolves-pending");

            Ok(())
        }
        /*
        Commands::Commit { message } => {
            let workspace =
                Workspace::find_in_current_directory(repository_index, provider).await?;

            match workspace.commit(&message, CommitMode::Strict).await {
                Ok(_) => Ok(()),
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
                        println!();
                    }

                    println!();

                    Err(anyhow::anyhow!("refusing to commit"))
                }
                Err(err) => Err(err.into()),
            }
        }
        */
        _ => Err(anyhow::anyhow!("todo")),
    }
}

/*
fn print_changes(
    workspace: &Workspace,
    stdout: &mut StandardStream,
    changes: &BTreeSet<Change>,
) -> anyhow::Result<()> {
    if !changes.is_empty() {
        let current_dir =
            std::env::current_dir().map_other_err("failed to determine current directory")?;

        println!("\nChanges:");

        for change in changes {
            if change.change_type().has_modifications() {
                stdout.set_color(&green())?;
            } else {
                stdout.set_color(&yellow())?;
            }

            print!(
                "\t{:>8}:   (todo)",
                change.change_type().to_human_string(),
                //workspace.make_relative_path(&current_dir, change.canonical_path()),
            );

            if !change.change_type().has_modifications() {
                stdout.reset()?;
                print!(" (no modifications staged yet)");
            }

            println!();
        }

        stdout.reset()?;
    }

    Ok(())
}
*/
