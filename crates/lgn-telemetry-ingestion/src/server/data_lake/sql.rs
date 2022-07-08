use anyhow::{Context, Result};
use lgn_tracing::prelude::*;
use sqlx::Executor;
use sqlx::Row;

pub const LATEST_SCHEMA_VERSION: i32 = 2;

pub async fn read_schema_version(connection: &mut sqlx::AnyConnection) -> i32 {
    match sqlx::query(
        "SELECT version
         FROM migration;",
    )
    .fetch_one(connection)
    .await
    {
        Ok(row) => row.get("version"),
        Err(e) => {
            info!("Error reading schema version, assuming version 0: {}", e);
            0
        }
    }
}

pub async fn upgrade_schema_v2(connection: &mut sqlx::AnyConnection) -> Result<()> {
    connection
        .execute("ALTER TABLE blocks ADD payload_size BIGINT;")
        .await
        .with_context(|| "Adding column payload_size to table blocks")?;
    connection
        .execute("UPDATE migration SET version=2;")
        .await
        .with_context(|| "Updating schema version to 2")?;
    Ok(())
}

pub async fn execute_migration(connection: &mut sqlx::AnyConnection) -> Result<()> {
    let mut current_version = read_schema_version(connection).await;
    if 0 == current_version {
        info!("creating v1 schema");
        create_tables(connection).await?;
        current_version = read_schema_version(connection).await;
    }
    if 1 == current_version {
        info!("upgrading schema to v2");
        upgrade_schema_v2(connection).await?;
        current_version = read_schema_version(connection).await;
    }
    assert_eq!(current_version, LATEST_SCHEMA_VERSION);
    Ok(())
}

pub async fn create_tables(connection: &mut sqlx::AnyConnection) -> Result<()> {
    create_processes_table(connection).await?;
    create_streams_table(connection).await?;
    create_blocks_table(connection).await?;
    create_payloads_table(connection).await?;
    create_migration_table(connection).await?;
    Ok(())
}

async fn create_migration_table(connection: &mut sqlx::AnyConnection) -> Result<()> {
    sqlx::query("CREATE table migration(version BIGINT);")
        .execute(&mut *connection)
        .await
        .with_context(|| String::from("Creating table migration"))?;
    sqlx::query("INSERT INTO migration VALUES(1);")
        .execute(connection)
        .await
        .with_context(|| String::from("Recording the initial schema version"))?;
    Ok(())
}

async fn create_processes_table(connection: &mut sqlx::AnyConnection) -> Result<()> {
    let sql = "
         CREATE TABLE processes(
                  process_id VARCHAR(36),
                  exe VARCHAR(255),
                  username VARCHAR(255),
                  realname VARCHAR(255),
                  computer VARCHAR(255),
                  distro VARCHAR(255),
                  cpu_brand VARCHAR(255),
                  tsc_frequency BIGINT,
                  start_time VARCHAR(255),
                  start_ticks BIGINT,
                  insert_date DATE,
                  parent_process_id VARCHAR(36));
         CREATE INDEX process_id on processes(process_id);
         CREATE INDEX parent_process_id on processes(parent_process_id);
         CREATE INDEX process_insert_date on processes(insert_date);";
    connection
        .execute(sql)
        .await
        .with_context(|| String::from("Creating table processes and its indices"))?;
    Ok(())
}

async fn create_streams_table(connection: &mut sqlx::AnyConnection) -> Result<()> {
    // storing tags as text is simplistic - we should move to a tags table if we
    // keep the telemetry metadata in a SQL db
    let sql = "
         CREATE TABLE streams(
                  stream_id VARCHAR(36),
                  process_id VARCHAR(36),
                  dependencies_metadata BLOB,
                  objects_metadata BLOB,
                  tags TEXT,
                  properties TEXT
                  );
         CREATE INDEX stream_id on streams(stream_id);
         CREATE INDEX stream_process_id on streams(process_id);";
    connection
        .execute(sql)
        .await
        .with_context(|| String::from("Creating table streams and its indices"))?;
    Ok(())
}

async fn create_blocks_table(connection: &mut sqlx::AnyConnection) -> Result<()> {
    let sql = "
         CREATE TABLE blocks(
                  block_id VARCHAR(36),
                  stream_id VARCHAR(36),
                  begin_time VARCHAR(255),
                  begin_ticks BIGINT,
                  end_time VARCHAR(255),
                  end_ticks BIGINT,
                  nb_objects INT
                  );
         CREATE INDEX block_id on blocks(block_id);
         CREATE INDEX block_stream_id on blocks(stream_id);";
    connection
        .execute(sql)
        .await
        .with_context(|| String::from("Creating table blocks and its indices"))?;
    Ok(())
}

async fn create_payloads_table(connection: &mut sqlx::AnyConnection) -> Result<()> {
    let sql = "
         CREATE TABLE payloads(
                  block_id VARCHAR(36),
                  payload LONGBLOB
                  );
         CREATE INDEX payload_block_id on payloads(block_id);";
    connection
        .execute(sql)
        .await
        .with_context(|| "Creating table payloads and its index")?;
    Ok(())
}
