use crate::read_binary_chunk;
use anyhow::Result;
use lgn_tracing::warn;
use lgn_tracing_transit::parse_string::parse_string;
use prost::Message;

/// The `Block` as sent by the instrumented applications.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub block_id: String,
    pub stream_id: String,
    pub begin_time: String, // RFC3339
    pub end_time: String,   // RFC3339
    /// We send both RFC3339 times and ticks to be able to calibrate the tick frequency
    pub begin_ticks: i64,
    pub end_ticks: i64,
    pub nb_objects: i32,
}

impl From<Block> for crate::api::components::Block {
    fn from(block: Block) -> Self {
        Self {
            block_id: block.block_id,
            stream_id: block.stream_id,
            begin_time: block.begin_time,
            end_time: block.end_time,
            begin_ticks: block.begin_ticks.to_string(),
            end_ticks: block.end_ticks.to_string(),
            nb_objects: block.nb_objects.to_string(),
        }
    }
}

impl TryFrom<crate::api::components::Block> for Block {
    type Error = anyhow::Error;

    fn try_from(block: crate::api::components::Block) -> Result<Self> {
        Ok(Self {
            block_id: block.block_id,
            stream_id: block.stream_id,
            begin_time: block.begin_time,
            end_time: block.end_time,
            begin_ticks: block.begin_ticks.parse()?,
            end_ticks: block.end_ticks.parse()?,
            nb_objects: block.nb_objects.parse()?,
        })
    }
}

/// The `BlockPayload` sent along with the `Block`.
#[derive(Clone, PartialEq, prost::Message)]
pub struct BlockPayload {
    #[prost(bytes = "vec", tag = "1")]
    pub dependencies: Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub objects: Vec<u8>,
}

// TODO: See if we want to keep the protobuf encoding or not.
impl BlockPayload {
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

/// The `BlockMedatada` saved in the database.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockMetadata {
    pub block_id: String,
    pub stream_id: String,
    pub begin_time: String,
    pub end_time: String,
    pub begin_ticks: i64,
    pub end_ticks: i64,
    pub nb_objects: i32,
    pub payload_size: i64,
}

impl From<BlockMetadata> for crate::api::components::BlockMetadata {
    fn from(block: BlockMetadata) -> Self {
        Self {
            block_id: block.block_id,
            stream_id: block.stream_id,
            begin_time: block.begin_time,
            end_time: block.end_time,
            begin_ticks: block.begin_ticks,
            end_ticks: block.end_ticks,
            nb_objects: block.nb_objects,
            payload_size: block.payload_size,
        }
    }
}

impl From<crate::api::components::BlockMetadata> for BlockMetadata {
    fn from(block: crate::api::components::BlockMetadata) -> Self {
        Self {
            block_id: block.block_id,
            stream_id: block.stream_id,
            begin_time: block.begin_time,
            end_time: block.end_time,
            begin_ticks: block.begin_ticks,
            end_ticks: block.end_ticks,
            nb_objects: block.nb_objects,
            payload_size: block.payload_size,
        }
    }
}

/// Encode a block information and its payload into a buffer.
///
/// # Errors
///
/// This function will return an error if encoding fails.
pub fn encode_block_and_payload(block: Block, payload: BlockPayload) -> Result<Vec<u8>> {
    // The encoded block must match the unreal client and only contain numbers as string.
    let block: crate::api::components::Block = block.into();

    let mut buffer = Vec::new();
    let block_bytes = serde_json::to_vec(&block)?;
    buffer.push(2); // utf8
    buffer.extend((block_bytes.len() as u32).to_le_bytes());
    buffer.extend(block_bytes);
    buffer.extend((payload.dependencies.len() as u32).to_le_bytes());
    buffer.extend(payload.dependencies);
    buffer.extend((payload.objects.len() as u32).to_le_bytes());
    buffer.extend(payload.objects);
    Ok(buffer)
}

/// Decode a block information and its payload from a buffer.
///
/// # Errors
///
/// This function will return an error if the decoding fails.
pub fn decode_block_and_payload(data: &[u8]) -> Result<(Block, BlockPayload)> {
    let mut offset = 0;
    let block_text = parse_string(data, &mut offset)?;
    let block: crate::api::components::Block = serde_json::from_str(&block_text)?;

    let dependencies = read_binary_chunk(data, &mut offset)?;
    let objects = read_binary_chunk(data, &mut offset)?;
    let payload = BlockPayload {
        dependencies,
        objects,
    };

    if offset != data.len() {
        warn!("decode_block_and_payload: data was not parsed completely");
    }

    Ok((block.try_into()?, payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let block = Block {
            block_id: "123".to_string(),
            stream_id: "456".to_string(),
            begin_time: "2020-01-01T00:00:00Z".to_string(),
            end_time: "2020-01-01T00:00:00Z".to_string(),
            begin_ticks: 1,
            end_ticks: 2,
            nb_objects: 3,
        };
        let payload = BlockPayload {
            dependencies: b"123".to_vec(),
            objects: b"456".to_vec(),
        };
        let data = encode_block_and_payload(block.clone(), payload.clone()).unwrap();

        let (block2, payload2) = decode_block_and_payload(&data).unwrap();
        assert_eq!(block, block2);
        assert_eq!(payload, payload2);
    }
}
