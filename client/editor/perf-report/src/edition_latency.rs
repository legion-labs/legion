use anyhow::{Context, Result};
use legion_analytics::prelude::*;
use std::path::Path;

async fn find_server_process_id(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    editor_client_process_id: &str,
) -> Result<String> {
    let re = regex::Regex::new(r"received control message\. msg=(?P<msg>\{[^\}]*})")
        .with_context(|| "find_server_process_id")?;
    let process_id = find_process_log_entry(
        connection,
        data_path,
        editor_client_process_id,
        |_time, entry| {
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
        },
    )
    .await?
    .with_context(|| "searching for hello control message with remote process id")?;
    Ok(process_id)
}

async fn find_client_edition_commands(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    editor_client_process_id: &str,
) -> Result<Vec<(u64, String)>> {
    let re = regex::Regex::new(r"sending edition_command=(?P<cmd>\{[^\}]*})")
        .with_context(|| "find_edition_commands")?;
    let mut res = vec![];
    for_each_process_log_entry(
        connection,
        data_path,
        editor_client_process_id,
        |time, entry| {
            if let Some(Ok(cmd)) = re
                .captures(&entry)
                .map(|captures| captures.name("cmd"))
                .flatten()
                .map(|mat| json::parse(mat.as_str()))
            {
                if let Some(command_id) = cmd["id"].as_str() {
                    res.push((time, String::from(command_id)));
                }
            }
        },
    )
    .await?;
    Ok(res)
}

async fn find_server_edition_commands(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    editor_server_process_id: &str,
) -> Result<Vec<(u64, String)>> {
    let re = regex::Regex::new(r"received \w* command id=(?P<id>.*)")
        .with_context(|| "find_edition_commands")?;
    let mut res = vec![];
    for_each_process_log_entry(
        connection,
        data_path,
        editor_server_process_id,
        |time, entry| {
            if let Some(command_id) = re
                .captures(&entry)
                .map(|captures| captures.name("id"))
                .flatten()
                .map(|mat| mat.as_str())
            {
                res.push((time, String::from(command_id)));
            }
        },
    )
    .await?;
    Ok(res)
}

pub async fn print_edition_latency(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    editor_client_process_id: &str,
) -> Result<()> {
    let server_process_id =
        find_server_process_id(connection, data_path, editor_client_process_id).await?;
    println!("server process id: {}", server_process_id);

    let edition_commands =
        find_client_edition_commands(connection, data_path, editor_client_process_id).await?;
    println!("\nclient commands:");
    for command in edition_commands {
        println!("{} {}", command.0, command.1);
    }
    let server_commands =
        find_server_edition_commands(connection, data_path, &server_process_id).await?;
    println!("\nserver commands:");
    for command in server_commands {
        println!("{} {}", command.0, command.1);
    }
    

    Ok(())
}
