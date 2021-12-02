use std::{collections::HashMap, path::Path};

use anyhow::{Context, Result};
use legion_analytics::prelude::*;
use legion_transit::prelude::*;

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

async fn find_process_metrics(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    editor_server_process_id: &str,
    metric_name: &str,
) -> Result<Vec<(u64, u64)>> {
    let mut res = vec![];
    for_each_process_metric(
        connection,
        data_path,
        editor_server_process_id,
        |metric_instance| {
            let metric_desc = metric_instance.get::<Object>("metric").unwrap();
            let name = metric_desc.get_ref("name").unwrap().as_str().unwrap();
            if name == metric_name {
                let time = metric_instance.get::<u64>("time").unwrap();
                let frame_id = metric_instance.get::<u64>("value").unwrap();
                res.push((time, frame_id));
            }
        },
    )
    .await?;
    Ok(res)
}

// TODO: Make all times relative to start of process
fn find_timed_event<T>(v: &[(u64, T)], time: u64) -> Option<(u64, T)>
where
    T: Clone,
{
    let index = v.partition_point(|item| item.0 < time);
    if index < v.len() {
        assert!(v[index].0 >= time);
        Some(v[index].clone())
    } else {
        None
    }
}

pub async fn print_edition_latency(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    editor_client_process_id: &str,
) -> Result<()> {
    let client_process_info = find_process(connection, editor_client_process_id).await?;
    let server_process_id =
        find_server_process_id(connection, data_path, editor_client_process_id).await?;
    println!("server process id: {}", server_process_id);

    let server_commands =
        find_server_edition_commands(connection, data_path, &server_process_id).await?;
    let mut server_command_timestamps = HashMap::new();
    for (time, uuid) in server_commands {
        server_command_timestamps.insert(uuid, time);
    }

    let server_frames = find_process_metrics(
        connection,
        data_path,
        &server_process_id,
        "Frame ID begin render",
    )
    .await?;

    let client_frames = find_process_metrics(
        connection,
        data_path,
        editor_client_process_id,
        "Frame ID of chunk received",
    )
    .await?;
    let mut client_frames_reception_timestamp = HashMap::new();
    for (time, frame_id) in client_frames {
        client_frames_reception_timestamp.insert(frame_id, time);
    }

    let start_client_process = client_process_info.start_ticks;
    dbg!(start_client_process);
    let edition_commands =
        find_client_edition_commands(connection, data_path, editor_client_process_id).await?;
    println!("\nclient command latencies:");
    for (client_command_timestamp, command_id) in &edition_commands {
        if let Some(server_command_reception_time) = server_command_timestamps.get(command_id) {
            if let Some((_time, frame_id)) =
                find_timed_event(&server_frames, *server_command_reception_time)
            {
                if let Some(client_reception_time) =
                    client_frames_reception_timestamp.get(&frame_id)
                {
                    let time_of_command = ((client_command_timestamp - start_client_process) * 1000)
                        as f64
                        / (client_process_info.tsc_frequency as f64);
                    let latency = ((client_reception_time - client_command_timestamp) * 1000)
                        as f64
                        / (client_process_info.tsc_frequency as f64);
                    println!("{},{}", time_of_command, latency);
                }
            }
        }
    }

    Ok(())
}
