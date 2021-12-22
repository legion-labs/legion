use std::sync::Arc;

use lgn_telemetry::{
    event_block::TelemetryBlock, log, EventSink, EventStream, LogBlock, MetricsBlock, ThreadBlock,
};
use lgn_telemetry_proto::{
    ingestion::telemetry_ingestion_client::TelemetryIngestionClient,
    telemetry::{
        ContainerMetadata, Process as ProcessInfo, Stream as StreamInfo, UdtMember, UserDefinedType,
    },
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
    sender: std::sync::mpsc::Sender<SinkEvent>,
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
        let (sender, receiver) = std::sync::mpsc::channel::<SinkEvent>();
        Self {
            thread: Some(std::thread::spawn(move || {
                Self::thread_proc(addr, receiver);
            })),
            sender,
        }
    }

    async fn thread_proc_impl(addr: String, receiver: std::sync::mpsc::Receiver<SinkEvent>) {
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
                    SinkEvent::ProcessLogBlock(buffer) => match buffer.encode() {
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
                    SinkEvent::ProcessMetricsBlock(buffer) => match buffer.encode() {
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
                    SinkEvent::ProcessThreadBlock(buffer) => match buffer.encode() {
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
                },
                Err(e) => {
                    println!("Error in telemetry thread: {}", e);
                    return;
                }
            }
        }
    }

    #[allow(clippy::needless_pass_by_value)] // we don't want to leave the receiver in the calling thread
    fn thread_proc(addr: String, receiver: std::sync::mpsc::Receiver<SinkEvent>) {
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
        tokio_runtime.block_on(Self::thread_proc_impl(addr, receiver));
    }
}

impl EventSink for GRPCEventSink {
    fn on_startup(&self, process_info: lgn_telemetry::ProcessInfo) {
        if let Err(e) = self.sender.send(SinkEvent::Startup(ProcessInfo {
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
        })) {
            dbg!(e);
        }
    }

    fn on_shutdown(&self) {
        // nothing to do
    }

    fn on_log_enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }

    fn on_log(&self, _: &log::Record<'_>) {}

    fn on_init_log_stream(&self, log_stream: &lgn_telemetry::LogStream) {
        if let Err(e) = self
            .sender
            .send(SinkEvent::InitStream(get_stream_info(log_stream)))
        {
            dbg!(e);
        }
    }

    fn on_process_log_block(&self, log_block: Arc<LogBlock>) {
        if let Err(e) = self.sender.send(SinkEvent::ProcessLogBlock(log_block)) {
            dbg!(e);
        }
    }

    fn on_init_metrics_stream(&self, metrics_stream: &lgn_telemetry::MetricsStream) {
        if let Err(e) = self
            .sender
            .send(SinkEvent::InitStream(get_stream_info(metrics_stream)))
        {
            dbg!(e);
        }
    }

    fn on_process_metrics_block(&self, metrics_block: Arc<MetricsBlock>) {
        if let Err(e) = self
            .sender
            .send(SinkEvent::ProcessMetricsBlock(metrics_block))
        {
            dbg!(e);
        }
    }

    fn on_init_thread_stream(&self, thread_stream: &lgn_telemetry::ThreadStream) {
        if let Err(e) = self
            .sender
            .send(SinkEvent::InitStream(get_stream_info(thread_stream)))
        {
            dbg!(e);
        }
    }

    fn on_process_thread_block(&self, thread_block: Arc<ThreadBlock>) {
        if let Err(e) = self
            .sender
            .send(SinkEvent::ProcessThreadBlock(thread_block))
        {
            dbg!(e);
        }
    }
}

fn get_stream_info<Block, DepsQueue>(stream: &EventStream<Block, DepsQueue>) -> StreamInfo
where
    Block: TelemetryBlock,
    DepsQueue: lgn_transit::HeterogeneousQueue,
    <Block as TelemetryBlock>::Queue: lgn_transit::HeterogeneousQueue,
{
    let dependencies_meta = make_queue_metedata::<DepsQueue>();
    let obj_meta = make_queue_metedata::<Block::Queue>();
    StreamInfo {
        process_id: stream.get_process_id(),
        stream_id: stream.stream_id().to_string(),
        dependencies_metadata: Some(dependencies_meta),
        objects_metadata: Some(obj_meta),
        tags: stream.get_tags(),
        properties: stream.get_properties(),
    }
}

fn make_queue_metedata<Queue: lgn_transit::HeterogeneousQueue>() -> ContainerMetadata {
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
