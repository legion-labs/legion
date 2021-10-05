use analytics::*;

pub async fn print_recent_processes(connection: &mut sqlx::AnyConnection) {
    for p in fetch_recent_processes(connection).await.unwrap() {
        println!("{} {} {}", p.start_time, p.process_id, p.exe);
    }
}
