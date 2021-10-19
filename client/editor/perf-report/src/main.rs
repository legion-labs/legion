use anyhow::{bail, Context, Result};
use clap::{App, AppSettings, Arg, SubCommand};
use legion_analytics::prelude::*;
use legion_telemetry::prelude::*;
use std::path::Path;

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
            print_edition_latency(&mut connection, data_path, process_id).await?;
        }
        (command_name, _args) => {
            log_str(LogLevel::Info, "unknown subcommand match");
            bail!("unknown subcommand match: {:?}", &command_name);
        }
    }
    Ok(())
}

async fn print_edition_latency(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    editor_client_process_id: &str,
) -> Result<()> {
    let re = regex::Regex::new(r"received control message\. msg=(?P<msg>\{[^\}]*})").unwrap();
    let process_id =
        find_process_log_entry(connection, data_path, editor_client_process_id, |entry| {
            if let Some(Ok(msg)) = re
                .captures(&entry)
                .map(|captures| captures.name("msg"))
                .flatten()
                .map(|mat| json::parse(mat.as_str()))
            {
                if msg["control_msg"] == "hello" {
                    if let Some(process_id) = msg["process_id"].as_str() {
                        return Some(process_id.to_owned());
                    }
                }
            }
            None
        })
        .await?
        .with_context(|| "searching for hello control message with remote process id")?;
    dbg!(process_id);
    Ok(())
}
