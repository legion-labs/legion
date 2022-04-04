use crate::data_lake_connection::DataLakeConnection;
use anyhow::Context;
use anyhow::Result;
use lgn_tracing::prelude::*;

#[derive(Clone)]
pub struct WebIngestionService {
    lake: DataLakeConnection,
}

impl WebIngestionService {
    pub fn new(lake: DataLakeConnection) -> Self {
        Self { lake }
    }

    #[span_fn]
    pub async fn insert_process(&self, body: serde_json::value::Value) -> Result<()> {
        let mut connection = self.lake.db_pool.acquire().await?;
        let current_date: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
        let tsc_frequency = body["tsc_frequency"]
            .as_str()
            .with_context(|| "reading field tsc_frequency")?
            .parse::<i64>()
            .with_context(|| "parsing tsc_frequency")?;

        let start_ticks = body["start_ticks"]
            .as_str()
            .with_context(|| "reading field start_ticks")?
            .parse::<i64>()
            .with_context(|| "parsing start_ticks")?;

        sqlx::query("INSERT INTO processes VALUES(?,?,?,?,?,?,?,?,?,?,?,?);")
            .bind(
                body["process_id"]
                    .as_str()
                    .with_context(|| "reading field process_id")?,
            )
            .bind(body["exe"].as_str().with_context(|| "reading field exe")?)
            .bind(
                body["username"]
                    .as_str()
                    .with_context(|| "reading field username")?,
            )
            .bind(
                body["realname"]
                    .as_str()
                    .with_context(|| "reading field realname")?,
            )
            .bind(
                body["computer"]
                    .as_str()
                    .with_context(|| "reading field computer")?,
            )
            .bind(
                body["distro"]
                    .as_str()
                    .with_context(|| "reading field distro")?,
            )
            .bind(
                body["cpu_brand"]
                    .as_str()
                    .with_context(|| "reading field cpu_brand")?,
            )
            .bind(tsc_frequency)
            .bind(
                body["start_time"]
                    .as_str()
                    .with_context(|| "reading field start_time")?,
            )
            .bind(start_ticks)
            .bind(current_date.format("%Y-%m-%d").to_string())
            .bind(
                body["parent_process_id"]
                    .as_str()
                    .with_context(|| "reading field parent_process_id")?,
            )
            .execute(&mut connection)
            .await
            .with_context(|| "executing sql insert into processes")?;
        Ok(())
    }
}
