use std::sync::atomic::{AtomicIsize, Ordering};
use std::{
    fmt,
    sync::{Arc, Mutex},
};

use lgn_telemetry_proto::{
    ingestion::telemetry_ingestion_client::TelemetryIngestionClient,
    telemetry::{
        ContainerMetadata, Process as ProcessInfo, Stream as StreamInfo, UdtMember, UserDefinedType,
    },
};
use lgn_tracing::{
    error,
    event::{EventSink, EventStream, ExtractDeps, TracingBlock},
    logs::{LogBlock, LogMetadata, LogStream},
    metrics::{MetricsBlock, MetricsStream},
    spans::{ThreadBlock, ThreadStream},
    Level,
};

use crate::stream::StreamBlock;

#[derive(Debug)]
enum SinkEvent {
    Startup(ProcessInfo),
    InitStream(StreamInfo),
    ProcessLogBlock(Arc<LogBlock>),
    ProcessMetricsBlock(Arc<MetricsBlock>),
    ProcessThreadBlock(Arc<ThreadBlock>),
}

pub struct GRPCEventSink {
    thread: Option<std::thread::JoinHandle<()>>,
    sender: Mutex<Option<std::sync::mpsc::Sender<SinkEvent>>>,
    queue_size: Arc<AtomicIsize>,
}

impl Drop for GRPCEventSink {
    fn drop(&mut self) {
        let mut sender_guard = self.sender.lock().unwrap();
        *sender_guard = None;
        if let Some(handle) = self.thread.take() {
            handle.join().expect("Error joining telemetry thread");
        }
    }
}

impl GRPCEventSink {
    pub fn new(addr_server: &str, max_queue_size: isize) -> Self {
        let addr = addr_server.to_owned();
        let (sender, receiver) = std::sync::mpsc::channel::<SinkEvent>();
        let queue_size = Arc::new(AtomicIsize::new(0));
        let thread_queue_size = queue_size.clone();
        Self {
            thread: Some(std::thread::spawn(move || {
                Self::thread_proc(addr, receiver, thread_queue_size, max_queue_size);
            })),
            sender: Mutex::new(Some(sender)),
            queue_size,
        }
    }

    fn send(&self, event: SinkEvent) {
        let guard = self.sender.lock().unwrap();
        if let Some(sender) = guard.as_ref() {
            self.queue_size.fetch_add(1, Ordering::Relaxed);
            if let Err(e) = sender.send(event) {
                self.queue_size.fetch_sub(1, Ordering::Relaxed);
                error!("{}", e);
            }
        }
    }

    async fn push_block(
        client: &mut TelemetryIngestionClient<tonic::transport::Channel>,
        buffer: &dyn StreamBlock,
        current_queue_size: &AtomicIsize,
        max_queue_size: isize,
    ) {
        if current_queue_size.load(Ordering::Relaxed) >= max_queue_size {
            // could be better to have a budget for each block type
            // this way thread data would not starve the other streams
            return;
        }
        match buffer.encode() {
            Ok(encoded_block) => match client.insert_block(encoded_block).await {
                Ok(_response) => {}
                Err(e) => {
                    println!("insert_block failed: {}", e);
                }
            },
            Err(e) => {
                println!("block encoding failed: {}", e);
            }
        }
    }

    async fn thread_proc_impl(
        addr: String,
        receiver: std::sync::mpsc::Receiver<SinkEvent>,
        queue_size: Arc<AtomicIsize>,
        max_queue_size: isize,
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
                    SinkEvent::Startup(process_info) => {
                        match client.insert_process(process_info).await {
                            Ok(_response) => {}
                            Err(e) => {
                                println!("insert_process failed: {}", e);
                            }
                        }
                    }
                    SinkEvent::InitStream(stream_info) => {
                        match client.insert_stream(stream_info).await {
                            Ok(_response) => {}
                            Err(e) => {
                                println!("insert_process failed: {}", e);
                            }
                        }
                    }
                    SinkEvent::ProcessLogBlock(buffer) => {
                        Self::push_block(&mut client, &*buffer, &*queue_size, max_queue_size).await;
                    }
                    SinkEvent::ProcessMetricsBlock(buffer) => {
                        Self::push_block(&mut client, &*buffer, &*queue_size, max_queue_size).await;
                    }
                    SinkEvent::ProcessThreadBlock(buffer) => {
                        Self::push_block(&mut client, &*buffer, &*queue_size, max_queue_size).await;
                    }
                },
                Err(_e) => {
                    // can only fail when the sending half is disconnected
                    // println!("Error in telemetry thread: {}", e);
                    return;
                }
            }
            queue_size.fetch_sub(1, Ordering::Relaxed);
        }
    }

    #[allow(clippy::needless_pass_by_value)] // we don't want to leave the receiver in the calling thread
    fn thread_proc(
        addr: String,
        receiver: std::sync::mpsc::Receiver<SinkEvent>,
        queue_size: Arc<AtomicIsize>,
        max_queue_size: isize,
    ) {
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
        tokio_runtime.block_on(Self::thread_proc_impl(
            addr,
            receiver,
            queue_size,
            max_queue_size,
        ));
    }
}

impl EventSink for GRPCEventSink {
    fn on_startup(&self, process_info: lgn_tracing::ProcessInfo) {
        self.send(SinkEvent::Startup(ProcessInfo {
            process_id: process_info.process_id,
            exe: process_info.exe,
            username: process_info.username,
            realname: process_info.realname,
            computer: process_info.computer,
            distro: process_info.distro,
            cpu_brand: process_info.cpu_brand,
            tsc_frequency: process_info.tsc_frequency,
            start_time: process_info.start_time,
            start_ticks: process_info.start_ticks,
            parent_process_id: process_info.parent_process_id,
        }));
    }

    fn on_shutdown(&self) {
        // nothing to do
    }

    fn on_log_enabled(&self, _: Level, _: &str) -> bool {
        true
    }

    fn on_log(&self, _desc: &LogMetadata, _time: i64, _args: &fmt::Arguments<'_>) {}

    fn on_init_log_stream(&self, log_stream: &LogStream) {
        self.send(SinkEvent::InitStream(get_stream_info(log_stream)));
    }

    fn on_process_log_block(&self, log_block: Arc<LogBlock>) {
        self.send(SinkEvent::ProcessLogBlock(log_block));
    }

    fn on_init_metrics_stream(&self, metrics_stream: &MetricsStream) {
        self.send(SinkEvent::InitStream(get_stream_info(metrics_stream)));
    }

    fn on_process_metrics_block(&self, metrics_block: Arc<MetricsBlock>) {
        self.send(SinkEvent::ProcessMetricsBlock(metrics_block));
    }

    fn on_init_thread_stream(&self, thread_stream: &ThreadStream) {
        self.send(SinkEvent::InitStream(get_stream_info(thread_stream)));
    }

    fn on_process_thread_block(&self, thread_block: Arc<ThreadBlock>) {
        self.send(SinkEvent::ProcessThreadBlock(thread_block));
    }
}

fn get_stream_info<Block>(stream: &EventStream<Block>) -> StreamInfo
where
    Block: TracingBlock,
    <Block as TracingBlock>::Queue: lgn_tracing_transit::HeterogeneousQueue,
    <<Block as TracingBlock>::Queue as ExtractDeps>::DepsQueue:
        lgn_tracing_transit::HeterogeneousQueue,
{
    let dependencies_meta =
        make_queue_metedata::<<<Block as TracingBlock>::Queue as ExtractDeps>::DepsQueue>();
    let obj_meta = make_queue_metedata::<Block::Queue>();
    StreamInfo {
        process_id: stream.process_id().to_owned(),
        stream_id: stream.stream_id().to_owned(),
        dependencies_metadata: Some(dependencies_meta),
        objects_metadata: Some(obj_meta),
        tags: stream.tags().to_owned(),
        properties: stream.properties().clone(),
    }
}

fn make_queue_metedata<Queue: lgn_tracing_transit::HeterogeneousQueue>() -> ContainerMetadata {
    let udts = Queue::reflect_contained();
    ContainerMetadata {
        types: udts
            .iter()
            .map(|udt| UserDefinedType {
                name: udt.name.clone(),
                size: udt.size as u32,
                members: udt
                    .members
                    .iter()
                    .map(|member| UdtMember {
                        name: member.name.clone(),
                        type_name: member.type_name.clone(),
                        offset: member.offset as u32,
                        size: member.size as u32,
                        is_reference: member.is_reference,
                    })
                    .collect(),
            })
            .collect(),
    }
}
