use anyhow::Context;
use legion_analytics::prelude::*;

pub async fn print_recent_processes(connection: &mut sqlx::AnyConnection) {
    for p in fetch_recent_processes(connection).await.unwrap() {
        println!("{} {} {}", p.start_time, p.process_id, p.exe);
    }
}

pub async fn print_process_search(connection: &mut sqlx::AnyConnection, filter: &str) {
    for p in processes_by_name_substring(connection, filter)
        .await
        .with_context(|| "print_process_search")
        .unwrap()
    {
        println!("{} {} {}", p.start_time, p.process_id, p.exe);
    }
}

#[async_recursion::async_recursion]
async fn print_process_tree_impl(
    connection: &mut sqlx::AnyConnection,
    root: &legion_telemetry::ProcessInfo,
    indent_level: u16,
) {
    println!(
        "{}{} {}",
        " ".repeat(indent_level as usize * 2),
        &root.process_id,
        &root.exe
    );

    for child_info in fetch_child_processes(connection, &root.process_id)
        .await
        .unwrap()
    {
        print_process_tree_impl(connection, &child_info, indent_level + 1).await;
    }
}

pub async fn print_process_tree(connection: &mut sqlx::AnyConnection, root_process_id: &str) {
    let root_process_info = find_process(connection, root_process_id).await.unwrap();
    print_process_tree_impl(connection, &root_process_info, 0).await;
}
