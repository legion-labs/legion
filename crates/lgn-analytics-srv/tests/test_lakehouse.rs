//! lakehouse query test
use std::sync::Arc;

use anyhow::Result;
use datafusion::execution::context::SessionContext;
use lgn_telemetry_sink::TelemetryGuard;

#[tokio::test]
#[ignore]
async fn test_lakehouse_query() -> Result<()> {
    let _telemetry_guard = TelemetryGuard::default().unwrap();
    let table_path = "d:/temp/cache/tables/3F5F22FF-445B-2156-96F6-3F8CA984968E/spans";
    let table = deltalake::open_table(table_path).await?;
    let ctx = SessionContext::new();
    ctx.register_table("spans", Arc::new(table))?;
    let _batches = ctx
        .sql("SELECT count(*) FROM spans where begin_ms > 5000")
        .await?
        .collect()
        .await?;
    //dbg!(batches);
    Ok(())
}
