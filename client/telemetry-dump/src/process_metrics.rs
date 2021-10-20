use anyhow::*;
use legion_analytics::*;
use std::path::Path;
use transit::*;

pub async fn print_process_metrics(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    process_id: &str,
) -> Result<()> {
    for_each_process_metric(connection, data_path, process_id, |obj| {
        let metric = obj.get::<Object>("metric").unwrap();
        let name = metric.get::<String>("name").unwrap();
        let unit = metric.get::<String>("unit").unwrap();
        let time = obj.get::<u64>("time").unwrap();
        if let Ok(int_value) = obj.get::<u64>("value") {
            println!("{} {} ({}) : {}", time, name, unit, int_value);
        } else if let Ok(float_value) = obj.get::<f64>("value") {
            println!("{} {} ({}) : {}", time, name, unit, float_value);
        }
    })
    .await?;
    Ok(())
}
