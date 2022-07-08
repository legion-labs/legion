//! telemetry protocols

#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::similar_names,
    clippy::use_self,
    clippy::return_self_not_must_use
)]

pub mod telemetry {
    tonic::include_proto!("telemetry");
}

pub mod health {
    tonic::include_proto!("health");
}

pub mod ingestion {
    tonic::include_proto!("ingestion");
}

pub mod analytics {
    tonic::include_proto!("analytics");
}
