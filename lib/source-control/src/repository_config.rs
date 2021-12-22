use anyhow::{Context, Result};

use crate::{sql::execute_sql, BlobStorageSpec};

pub async fn init_config_database(sql_connection: &mut sqlx::AnyConnection) -> Result<()> {
    let sql = "CREATE TABLE config(self_uri TEXT, blob_storage_spec TEXT);";

    execute_sql(sql_connection, sql)
        .await
        .context("error creating commit tables and indices")
}

pub async fn insert_config(
    sql_connection: &mut sqlx::AnyConnection,
    self_uri: &str,
    blob_storage: &BlobStorageSpec,
) -> Result<()> {
    sqlx::query("INSERT INTO config VALUES(?, ?);")
        .bind(self_uri)
        .bind(blob_storage.to_json())
        .execute(&mut *sql_connection)
        .await
        .context("error inserting into config")?;

    Ok(())
}
