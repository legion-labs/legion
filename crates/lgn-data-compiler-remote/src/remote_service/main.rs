use clap::StructOpt;
use lgn_tracing::LevelFilter;

fn main() {
    let _telemetry_guard = lgn_telemetry_sink::TelemetryGuard::default()
        .unwrap()
        .with_log_level(LevelFilter::Info);

    let options =
        lgn_data_compiler_remote::remote_service::common_types::RemoteExecutionArgs::parse();

    if options.server {
        lgn_data_compiler_remote::remote_service::service::run_server(&options);
    } else {
        lgn_data_compiler_remote::remote_service::worker::run_worker(options);
    }
}
