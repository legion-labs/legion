use std::env;

use lgn_compiler_gltf::COMPILER_INFO;
use lgn_data_compiler::compiler_api::{compiler_main, CompilerError};
use lgn_telemetry_sink::TelemetryGuardBuilder;

#[tokio::main]
async fn main() -> Result<(), CompilerError> {
    let _telemetry_guard = TelemetryGuardBuilder::default()
        .with_ctrlc_handling()
        .with_local_sink_enabled(false)
        .build();
    lgn_tracing::span_scope!("compiler-gltf::main");
    compiler_main(&env::args(), &COMPILER_INFO).await
}
