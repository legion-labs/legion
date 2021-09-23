use telemetry::telemetry_ingestion_proto::telemetry_ingestion_client::TelemetryIngestionClient;
use telemetry::telemetry_ingestion_proto::InsertProcessRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = TelemetryIngestionClient::connect("http://127.0.0.1:8080").await?;
    let id = uuid::Uuid::new_v4().to_string();
    let request = tonic::Request::new(InsertProcessRequest {
        id,
        username: "mad".into(),
        exe: "allo.exe".into(),
        computer: "trs-80".into(),
        tsc_frequency: 0,
    });
    let response = client.insert_process(request).await?;
    dbg!(response);
    Ok(())
}
