use std::collections::HashMap;

use lgn_telemetry::types::{
    ContainerMetadata, Stream as StreamInfo, UdtMember as UdtMemberInfo,
    UserDefinedType as UserDefinedTypeInfo,
};
use lgn_tracing::event::{EventStream, ExtractDeps, TracingBlock};
use lgn_tracing_transit::UserDefinedType;

pub fn get_stream_info<Block>(stream: &EventStream<Block>) -> StreamInfo
where
    Block: TracingBlock,
    <Block as TracingBlock>::Queue: lgn_tracing_transit::HeterogeneousQueue,
    <<Block as TracingBlock>::Queue as ExtractDeps>::DepsQueue:
        lgn_tracing_transit::HeterogeneousQueue,
{
    let dependencies_meta =
        make_queue_metadata::<<<Block as TracingBlock>::Queue as ExtractDeps>::DepsQueue>();
    let obj_meta = make_queue_metadata::<Block::Queue>();
    StreamInfo {
        process_id: stream.process_id().to_owned(),
        stream_id: stream.stream_id().to_owned(),
        dependencies_metadata: Some(dependencies_meta),
        objects_metadata: Some(obj_meta),
        tags: stream.tags().to_owned(),
        properties: stream.properties().clone(),
    }
}

fn proto_from_udt(
    secondary_types: &mut HashMap<String, UserDefinedTypeInfo>,
    udt: &UserDefinedType,
) -> UserDefinedTypeInfo {
    for secondary in &udt.secondary_udts {
        let sec_proto = proto_from_udt(secondary_types, secondary);
        secondary_types.insert(sec_proto.name.clone(), sec_proto);
    }
    UserDefinedTypeInfo {
        name: udt.name.clone(),
        size: udt.size as u32,
        members: udt
            .members
            .iter()
            .map(|member| UdtMemberInfo {
                name: member.name.clone(),
                type_name: member.type_name.clone(),
                offset: member.offset as u32,
                size: member.size as u32,
                is_reference: member.is_reference,
            })
            .collect(),
        is_reference: udt.is_reference,
    }
}

fn make_queue_metadata<Queue: lgn_tracing_transit::HeterogeneousQueue>() -> ContainerMetadata {
    let udts = Queue::reflect_contained();
    let mut secondary_types = HashMap::new();
    let mut types: Vec<UserDefinedTypeInfo> = udts
        .iter()
        .map(|udt| proto_from_udt(&mut secondary_types, udt))
        .collect();
    for (_k, v) in secondary_types {
        types.push(v);
    }
    ContainerMetadata { types }
}
