use std::{fs, path::PathBuf};

use clap::{AppSettings, Arg, SubCommand};
use lgn_telemetry::LevelFilter;

pub(crate) struct Config {
    pub root: PathBuf,
    pub log_level: LevelFilter,
    pub command_config: CommandConfig,
}

pub(crate) enum CommandConfig {
    Run,
}

const ARG_NAME_ROOT: &str = "root";
const ARG_NAME_DEBUG: &str = "debug";

const SUBCOMMAND_NAME_RUN: &str = "run";

impl Config {
    pub(crate) fn new() -> anyhow::Result<Self> {
        let args = clap::App::new("Legion Labs Virtual-Machine Agent")
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .about(
            "The Virtual-Machine Agent.",
        )
        .long_about(
            "The Virtual-Machine Agent (VM-Agent) that provisions and orchestrates the different components that compose a Legion Labs virtual-machine instance.",
        )
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::with_name(ARG_NAME_ROOT)
                .long(ARG_NAME_ROOT)
                .short("C")
                .takes_value(true)
                .help("The root path where the VM-Agent will look for all necessary executables and deployment resources."),
        )
        .arg(
            Arg::with_name(ARG_NAME_DEBUG)
                .long(ARG_NAME_DEBUG)
                .required(false)
                .short("d")
                .help("Enable debug output."),
        )
        .subcommand(
            SubCommand::with_name(SUBCOMMAND_NAME_RUN)
            .about("Run the VM-Agent locally.")
        )
        .get_matches();

        Ok(Self {
            root: fs::canonicalize(args.value_of(ARG_NAME_ROOT).unwrap_or("."))?,
            log_level: if args.is_present(ARG_NAME_DEBUG) {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            },
            command_config: match args.subcommand() {
                (SUBCOMMAND_NAME_RUN, Some(_args)) => Ok(CommandConfig::Run),
                _ => Err(anyhow::format_err!(
                    "no sub-command was specified.\n\n{}",
                    args.usage(),
                )),
            }?,
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
