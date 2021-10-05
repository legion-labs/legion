use crate::*;

pub fn make_queue_metedata<Queue: transit::ReflectiveQueue>(
) -> telemetry_ingestion_proto::ContainerMetadata {
    let udts = Queue::reflect_contained();
    ContainerMetadata::from(&*udts)
}
