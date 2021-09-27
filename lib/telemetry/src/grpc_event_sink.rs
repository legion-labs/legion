use crate::telemetry_ingestion_proto::telemetry_ingestion_client::TelemetryIngestionClient;
use crate::*;

pub struct GRPCEventSink {
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
    pub fn new(addr_server: &str) -> Self {
        let addr = addr_server.to_owned();
        let (sender, receiver) = std::sync::mpsc::channel::<TelemetrySinkEvent>();
        Self {
            thread: Some(std::thread::spawn(move || {
                Self::thread_proc(addr, receiver);
            })),
            sender,
        }
    }

    #[allow(clippy::needless_pass_by_value)] // we don't want to leave the receiver in the calling thread
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
                    TelemetrySinkEvent::OnInitStream(stream_info) => {
                        match tokio_runtime.block_on(client.insert_stream(stream_info)) {
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
                        let encoded = log_buffer.encode();
                        dbg!(encoded);
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

impl EventBlockSink for GRPCEventSink {
    fn on_sink_event(&self, event: TelemetrySinkEvent) {
        if let Err(e) = self.sender.send(event) {
            dbg!(e);
        }
    }
}
