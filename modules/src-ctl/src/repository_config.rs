use crate::{sql::*, *};
use futures::executor::block_on;

pub async fn init_config_database(sql_connection: &mut sqlx::AnyConnection) -> Result<(), String> {
    let sql = "CREATE TABLE config(self_uri TEXT, blob_storage_spec TEXT);";
    if let Err(e) = execute_sql(sql_connection, sql).await {
        return Err(format!("Error creating commit tables and indices: {}", e));
    }
    Ok(())
}

pub fn insert_config(
    sql_connection: &mut sqlx::AnyConnection,
    self_uri: &str,
    blob_storage: &BlobStorageSpec,
) -> Result<(), String> {
    if let Err(e) = block_on(
        sqlx::query("INSERT INTO config VALUES(?, ?);")
            .bind(self_uri)
            .bind(blob_storage.to_json())
            .execute(&mut *sql_connection),
    ) {
        return Err(format!("Error inserting into config: {}", e));
    }
    Ok(())
}
