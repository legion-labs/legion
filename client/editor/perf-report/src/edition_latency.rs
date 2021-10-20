use anyhow::{Context, Result};
use legion_analytics::prelude::*;
use std::path::Path;
use transit::prelude::*;

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

async fn find_server_begin_frame_metrics(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    editor_server_process_id: &str,
) -> Result<Vec<(u64, u64)>> {
    let mut res = vec![];
    for_each_process_metric(
        connection,
        data_path,
        editor_server_process_id,
        |metric_instance| {
            let metric_desc = metric_instance.get::<Object>("metric").unwrap();
            let name = metric_desc.get_ref("name").unwrap().as_str().unwrap();
            if name == "Frame ID begin render" {
                let time = metric_instance.get::<u64>("time").unwrap();
                let frame_id = metric_instance.get::<u64>("value").unwrap();
                res.push((time, frame_id));
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

    let server_frames =
        find_server_begin_frame_metrics(connection, data_path, &server_process_id).await?;
    for frame in server_frames {
        println!("{} {}", frame.0, frame.1);
    }

    Ok(())
}
