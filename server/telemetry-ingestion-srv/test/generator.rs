use std::sync::Arc;
use telemetry::*;
use telemetry_ingestion_proto::telemetry_ingestion_client::TelemetryIngestionClient;

enum TelemetrySinkMessage {
    OnInitProcess(ProcessInfo),
    OnShutdown,
}

struct GRPCEventSink {
    thread: Option<std::thread::JoinHandle<()>>,
    sender: std::sync::mpsc::Sender<TelemetrySinkMessage>,
}

impl Drop for GRPCEventSink {
    fn drop(&mut self) {
        if let Some(handle) = self.thread.take() {
            handle.join().expect("Error joining telemetry thread");
        }
    }
}

impl GRPCEventSink {
    fn thread_proc(addr: String, receiver: std::sync::mpsc::Receiver<TelemetrySinkMessage>) {
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
                    TelemetrySinkMessage::OnInitProcess(process_info) => {
                        match tokio_runtime.block_on(client.insert_process(process_info)) {
                            Ok(response) => {
                                dbg!(response);
                            }
                            Err(e) => {
                                println!("insert_process failed: {}", e);
                            }
                        }
                    }
                    TelemetrySinkMessage::OnShutdown => {
                        return;
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
    let (sender, receiver) = std::sync::mpsc::channel::<TelemetrySinkMessage>();
    Arc::new(GRPCEventSink {
        thread: Some(std::thread::spawn(move || {
            GRPCEventSink::thread_proc(addr, receiver)
        })),
        sender,
    })
}

impl EventBlockSink for GRPCEventSink {
    fn on_init_process(&self, process_info: ProcessInfo) {
        if let Err(e) = self
            .sender
            .send(TelemetrySinkMessage::OnInitProcess(process_info))
        {
            dbg!(e);
        }
    }

    fn on_log_buffer_full(&self, log_block: &LogMsgBlock) {
        println!("log buffer full: {} bytes", log_block.events.len_bytes());
    }

    fn on_thread_buffer_full(&self, thread_block: &ThreadEventBlock) {
        println!(
            "thread buffer full: {} bytes",
            thread_block.events.len_bytes()
        );
    }

    fn on_shutdown(&self) {
        if let Err(e) = self.sender.send(TelemetrySinkMessage::OnShutdown) {
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
