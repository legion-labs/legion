use std::sync::Arc;
use telemetry::*;
use telemetry_ingestion_proto::telemetry_ingestion_client::TelemetryIngestionClient;

struct GRPCEventSink {
    thread: Option<std::thread::JoinHandle<()>>,
    sender: std::sync::mpsc::Sender<TelemetrySinkEvent>,
}

impl Drop for GRPCEventSink {
    fn drop(&mut self) {
        if let Some(handle) = self.thread.take() {
            handle.join().expect("Error joining telemetry thread");
        }
    }
}

impl GRPCEventSink {
    fn thread_proc(addr: String, receiver: std::sync::mpsc::Receiver<TelemetrySinkEvent>) {
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
        let mut client = match tokio_runtime.block_on(TelemetryIngestionClient::connect(addr)) {
            Ok(c) => c,
            Err(e) => {
                println!("Error connecting to telemetry server: {}", e);
                return;
            }
        };

        loop {
            match receiver.recv() {
                Ok(message) => match message {
                    TelemetrySinkEvent::OnInitProcess(process_info) => {
                        match tokio_runtime.block_on(client.insert_process(process_info)) {
                            Ok(response) => {
                                dbg!(response);
                            }
                            Err(e) => {
                                println!("insert_process failed: {}", e);
                            }
                        }
                    }
                    TelemetrySinkEvent::OnShutdown => {
                        return;
                    }
                    TelemetrySinkEvent::OnLogBufferFull(log_buffer) => {
                        dbg!(log_buffer);
                    }
                    TelemetrySinkEvent::OnThreadBufferFull(thread_buffer) => {
                        dbg!(thread_buffer);
                    }
                },
                Err(e) => {
                    println!("Error in telemetry thread: {}", e);
                    return;
                }
            }
        }
    }
}

pub fn make_telemetry_connection(addr_server: &str) -> Arc<dyn EventBlockSink> {
    let addr = addr_server.to_owned();
    let (sender, receiver) = std::sync::mpsc::channel::<TelemetrySinkEvent>();
    Arc::new(GRPCEventSink {
        thread: Some(std::thread::spawn(move || {
            GRPCEventSink::thread_proc(addr, receiver)
        })),
        sender,
    })
}

impl EventBlockSink for GRPCEventSink {
    fn on_sink_event(&self, event: TelemetrySinkEvent) {
        if let Err(e) = self.sender.send(event) {
            dbg!(e);
        }
    }
}

fn init_telemetry() {
    let sink = make_telemetry_connection("http://127.0.0.1:8080");
    init_event_dispatch(1024, 1024 * 1024, sink).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_telemetry();
    shutdown_event_dispatch();
    Ok(())
}
