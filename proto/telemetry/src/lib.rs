//! telemetry protocols
//!

pub mod telemetry {
    tonic::include_proto!("telemetry");
}

pub mod ingestion {
    tonic::include_proto!("ingestion");
}

pub mod analytics{
    tonic::include_proto!("analytics");
}
