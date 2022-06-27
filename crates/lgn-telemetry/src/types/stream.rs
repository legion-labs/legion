use anyhow::Result;
use prost::Message;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    pub stream_id: String,
    pub process_id: String,
    pub dependencies_metadata: Option<ContainerMetadata>,
    pub objects_metadata: Option<ContainerMetadata>,
    pub tags: Vec<String>,
    pub properties: HashMap<String, String>,
}

impl From<crate::api::components::Stream> for Stream {
    fn from(stream: crate::api::components::Stream) -> Self {
        Self {
            stream_id: stream.stream_id,
            process_id: stream.process_id,
            dependencies_metadata: stream.dependencies_metadata.map(Into::into),
            objects_metadata: stream.objects_metadata.map(Into::into),
            tags: stream.tags,
            properties: stream.__additional_properties.into_iter().collect(),
        }
    }
}

impl From<Stream> for crate::api::components::Stream {
    fn from(stream: Stream) -> Self {
        Self {
            stream_id: stream.stream_id,
            process_id: stream.process_id,
            dependencies_metadata: stream.dependencies_metadata.map(Into::into),
            objects_metadata: stream.objects_metadata.map(Into::into),
            tags: stream.tags,
            __additional_properties: stream.properties.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContainerMetadata {
    pub types: Vec<UserDefinedType>,
}

// TODO: See if we want to keep the protobuf encoding or not.
impl ContainerMetadata {
    /// Decodes a bytes buffer into a `BlockPayload` using protobuf.
    ///
    /// # Errors
    ///
    /// This function will return an error if the decoding fails.
    pub fn decode(buffer: &[u8]) -> Result<Self> {
        Ok(lgn_telemetry_proto::telemetry::ContainerMetadata::decode(buffer)?.into())
    }
    pub fn encode(self) -> Vec<u8> {
        let metadata: lgn_telemetry_proto::telemetry::ContainerMetadata = self.into();
        metadata.encode_to_vec()
    }
}

impl From<crate::api::components::ContainerMetadata> for ContainerMetadata {
    fn from(metadata: crate::api::components::ContainerMetadata) -> Self {
        Self {
            types: metadata.types.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ContainerMetadata> for crate::api::components::ContainerMetadata {
    fn from(metadata: ContainerMetadata) -> Self {
        Self {
            types: metadata.types.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ContainerMetadata> for lgn_telemetry_proto::telemetry::ContainerMetadata {
    fn from(metadata: ContainerMetadata) -> Self {
        Self {
            types: metadata.types.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<lgn_telemetry_proto::telemetry::ContainerMetadata> for ContainerMetadata {
    fn from(metadata: lgn_telemetry_proto::telemetry::ContainerMetadata) -> Self {
        Self {
            types: metadata.types.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserDefinedType {
    pub name: String,
    pub size: u32,
    pub members: Vec<UdtMember>,
    pub is_reference: bool,
}

impl From<crate::api::components::UserDefinedType> for UserDefinedType {
    fn from(type_: crate::api::components::UserDefinedType) -> Self {
        Self {
            name: type_.name,
            size: type_.size,
            members: type_.members.into_iter().map(Into::into).collect(),
            is_reference: type_.is_reference,
        }
    }
}

impl From<UserDefinedType> for crate::api::components::UserDefinedType {
    fn from(type_: UserDefinedType) -> Self {
        Self {
            name: type_.name,
            size: type_.size,
            members: type_.members.into_iter().map(Into::into).collect(),
            is_reference: type_.is_reference,
        }
    }
}

impl From<UserDefinedType> for lgn_telemetry_proto::telemetry::UserDefinedType {
    fn from(type_: UserDefinedType) -> Self {
        Self {
            name: type_.name,
            size: type_.size,
            members: type_.members.into_iter().map(Into::into).collect(),
            is_reference: type_.is_reference,
        }
    }
}

impl From<lgn_telemetry_proto::telemetry::UserDefinedType> for UserDefinedType {
    fn from(type_: lgn_telemetry_proto::telemetry::UserDefinedType) -> Self {
        Self {
            name: type_.name,
            size: type_.size,
            members: type_.members.into_iter().map(Into::into).collect(),
            is_reference: type_.is_reference,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UdtMember {
    pub name: String,
    pub type_name: String,
    pub offset: u32,
    pub size: u32,
    pub is_reference: bool,
}

impl From<crate::api::components::UdtMember> for UdtMember {
    fn from(member: crate::api::components::UdtMember) -> Self {
        Self {
            name: member.name,
            type_name: member.type_name,
            offset: member.offset,
            size: member.size,
            is_reference: member.is_reference,
        }
    }
}

impl From<UdtMember> for crate::api::components::UdtMember {
    fn from(member: UdtMember) -> Self {
        Self {
            name: member.name,
            type_name: member.type_name,
            offset: member.offset,
            size: member.size,
            is_reference: member.is_reference,
        }
    }
}

impl From<UdtMember> for lgn_telemetry_proto::telemetry::UdtMember {
    fn from(member: UdtMember) -> Self {
        Self {
            name: member.name,
            type_name: member.type_name,
            offset: member.offset,
            size: member.size,
            is_reference: member.is_reference,
        }
    }
}

impl From<lgn_telemetry_proto::telemetry::UdtMember> for UdtMember {
    fn from(member: lgn_telemetry_proto::telemetry::UdtMember) -> Self {
        Self {
            name: member.name,
            type_name: member.type_name,
            offset: member.offset,
            size: member.size,
            is_reference: member.is_reference,
        }
    }
}
