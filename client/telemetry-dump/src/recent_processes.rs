use legion_analytics::prelude::*;

pub async fn print_recent_processes(connection: &mut sqlx::AnyConnection) {
    for p in fetch_recent_processes(connection).await.unwrap() {
        println!("{} {} {}", p.start_time, p.process_id, p.exe);
    }
}

pub async fn print_process_search(connection: &mut sqlx::AnyConnection, filter: &str) {
    for p in processes_by_name_substring(connection, filter)
        .await
        .unwrap()
    {
        println!("{} {} {}", p.start_time, p.process_id, p.exe);
    }
}
