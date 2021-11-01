use legion_telemetry_proto::telemetry::{ContainerMetadata, UdtMember, UserDefinedType};

pub fn make_queue_metedata<Queue: transit::HeterogeneousQueue>() -> ContainerMetadata {
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
