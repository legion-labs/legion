use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use lgn_tracing::LevelFilter;

pub(crate) struct Config {
    pub root: PathBuf,
    pub log_level: LevelFilter,
    pub command_config: CommandConfig,
}

pub(crate) enum CommandConfig {
    Run,
}

#[derive(Parser, Debug)]
#[clap(name = "Legion Labs Virtual-Machine Agent")]
#[clap(about = "The Virtual-Machine Agent.", version, author)]
#[clap(
    long_about = "The Virtual-Machine Agent (VM-Agent) that provisions and orchestrates the different components that compose a Legion Labs virtual-machine instance."
)]
#[clap(arg_required_else_help(true))]
struct Cli {
    /// The root path where the VM-Agent will look for all necessary executables and deployment resources.
    #[clap(short = 'C', long, default_value = ".")]
    pub root: PathBuf,
    /// Enable debug output.
    #[clap(short, long)]
    pub debug: bool,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Prints a list of recent processes
    #[clap(name = "run")]
    Run,
}

impl Config {
    pub(crate) fn new() -> anyhow::Result<Self> {
        let args = Cli::parse();
        Ok(Self {
            root: fs::canonicalize(args.root)?,
            log_level: if args.debug {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            },
            command_config: match args.command {
                Commands::Run => CommandConfig::Run,
            },
        })
    }

    pub fn editor_server_bin_path(&self) -> PathBuf {
        to_executable_name(self.root.join("editor-srv"))
    }
}

#[cfg(target_os = "windows")]
#[allow(clippy::needless_pass_by_value)]
fn to_executable_name(p: PathBuf) -> PathBuf {
    p.with_extension("exe")
}

#[cfg(not(target_os = "windows"))]
fn to_executable_name(p: PathBuf) -> PathBuf {
    p
}
