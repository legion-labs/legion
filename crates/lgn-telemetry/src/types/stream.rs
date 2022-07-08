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

impl TryFrom<crate::api::components::Stream> for Stream {
    type Error = anyhow::Error;

    fn try_from(stream: crate::api::components::Stream) -> Result<Self> {
        Ok(Self {
            stream_id: stream.stream_id,
            process_id: stream.process_id,
            dependencies_metadata: match stream.dependencies_metadata {
                Some(metadata) => Some(ContainerMetadata {
                    types: metadata
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<_>>()?,
                }),
                None => None,
            },
            objects_metadata: match stream.objects_metadata {
                Some(metadata) => Some(ContainerMetadata {
                    types: metadata
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<_>>()?,
                }),
                None => None,
            },
            tags: stream.tags,
            properties: stream.__additional_properties.into_iter().collect(),
        })
    }
}

impl From<Stream> for crate::api::components::Stream {
    fn from(stream: Stream) -> Self {
        Self {
            stream_id: stream.stream_id,
            process_id: stream.process_id,
            dependencies_metadata: stream
                .dependencies_metadata
                .map(|m| m.types.into_iter().map(Into::into).collect()),
            objects_metadata: stream
                .objects_metadata
                .map(|m| m.types.into_iter().map(Into::into).collect()),
            tags: stream.tags,
            __additional_properties: stream.properties.into_iter().collect(),
        }
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ContainerMetadata {
    #[prost(message, repeated, tag = "1")]
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
        Ok(Message::decode(buffer)?)
    }

    pub fn encode(self) -> Vec<u8> {
        self.encode_to_vec()
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct UserDefinedType {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(uint32, tag = "2")]
    pub size: u32,
    #[prost(message, repeated, tag = "3")]
    pub members: Vec<UdtMember>,
    #[prost(bool, tag = "4")]
    pub is_reference: bool,
}

impl TryFrom<crate::api::components::UserDefinedType> for UserDefinedType {
    type Error = anyhow::Error;

    fn try_from(type_: crate::api::components::UserDefinedType) -> Result<Self> {
        Ok(Self {
            name: type_.name,
            size: type_.size.parse()?,
            members: type_
                .members
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_>>()?,
            is_reference: type_.is_reference,
        })
    }
}

impl From<UserDefinedType> for crate::api::components::UserDefinedType {
    fn from(type_: UserDefinedType) -> Self {
        Self {
            name: type_.name,
            size: type_.size.to_string(),
            members: type_.members.into_iter().map(Into::into).collect(),
            is_reference: type_.is_reference,
        }
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct UdtMember {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(string, tag = "2")]
    pub type_name: String,
    #[prost(uint32, tag = "3")]
    pub offset: u32,
    #[prost(uint32, tag = "4")]
    pub size: u32,
    #[prost(bool, tag = "5")]
    pub is_reference: bool,
}

impl TryFrom<crate::api::components::UdtMember> for UdtMember {
    type Error = anyhow::Error;

    fn try_from(member: crate::api::components::UdtMember) -> Result<Self> {
        Ok(Self {
            name: member.name,
            type_name: member.type_name,
            offset: member.offset.parse()?,
            size: member.size.parse()?,
            is_reference: member.is_reference,
        })
    }
}

impl From<UdtMember> for crate::api::components::UdtMember {
    fn from(member: UdtMember) -> Self {
        Self {
            name: member.name,
            type_name: member.type_name,
            offset: member.offset.to_string(),
            size: member.size.to_string(),
            is_reference: member.is_reference,
        }
    }
}
