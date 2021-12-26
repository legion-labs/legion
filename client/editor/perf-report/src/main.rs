//! Perf report generation
//!

mod edition_latency;
use std::path::Path;

use anyhow::{bail, Result};
use clap::{App, AppSettings, Arg, SubCommand};
use lgn_analytics::prelude::*;
use lgn_telemetry::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("Legion Editor Performance Report")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("db")
                .required(true)
                .help("local path to folder containing telemetry.db3"),
        )
        .subcommand(
            SubCommand::with_name("edition-latency")
                .about("latency between a user command and receiving the corresponding video frame")
                .arg(
                    Arg::with_name("process-id")
                        .required(true)
                        .help("process guid"),
                ),
        )
        .get_matches();

    let data_path = Path::new(matches.value_of("db").unwrap());
    let pool = alloc_sql_pool(data_path).await.unwrap();
    let mut connection = pool.acquire().await.unwrap();
    match matches.subcommand() {
        ("edition-latency", Some(command_match)) => {
            let process_id = command_match.value_of("process-id").unwrap();
            edition_latency::print_edition_latency(&mut connection, data_path, process_id).await?;
        }
        (command_name, _args) => {
            info!("unknown subcommand match");
            bail!("unknown subcommand match: {:?}", &command_name);
        }
    }
    Ok(())
}
