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

    async fn thread_proc_impl(
        addr: String,
        receiver: std::sync::mpsc::Receiver<TelemetrySinkEvent>,
    ) {
        let mut client = match TelemetryIngestionClient::connect(addr).await {
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
                        match client.insert_process(process_info).await {
                            Ok(_response) => {}
                            Err(e) => {
                                println!("insert_process failed: {}", e);
                            }
                        }
                    }
                    TelemetrySinkEvent::OnInitStream(stream_info) => {
                        match client.insert_stream(stream_info).await {
                            Ok(_response) => {}
                            Err(e) => {
                                println!("insert_process failed: {}", e);
                            }
                        }
                    }
                    TelemetrySinkEvent::OnLogBufferFull(buffer) => match buffer.encode() {
                        Ok(encoded_block) => match client.insert_block(encoded_block).await {
                            Ok(_response) => {}
                            Err(e) => {
                                println!("insert_block failed: {}", e);
                            }
                        },
                        Err(e) => {
                            println!("block encoding failed: {}", e);
                        }
                    },
                    TelemetrySinkEvent::OnThreadBufferFull(buffer) => match buffer.encode() {
                        Ok(encoded_block) => match client.insert_block(encoded_block).await {
                            Ok(_response) => {}
                            Err(e) => {
                                println!("insert_block failed: {}", e);
                            }
                        },
                        Err(e) => {
                            println!("block encoding failed: {}", e);
                        }
                    },
                    TelemetrySinkEvent::OnShutdown => {
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

    #[allow(clippy::needless_pass_by_value)] // we don't want to leave the receiver in the calling thread
    fn thread_proc(addr: String, receiver: std::sync::mpsc::Receiver<TelemetrySinkEvent>) {
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
        tokio_runtime.block_on(Self::thread_proc_impl(addr, receiver));
    }
}

impl EventBlockSink for GRPCEventSink {
    fn on_sink_event(&self, event: TelemetrySinkEvent) {
        if let Err(e) = self.sender.send(event) {
            dbg!(e);
        }
    }
}
