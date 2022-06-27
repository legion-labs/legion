use crate::read_binary_chunk;
use anyhow::Result;
use lgn_tracing::warn;
use lgn_tracing_transit::parse_string::parse_string;

/// The `Block` as sent by the instrumented applications.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub block_id: String,
    pub stream_id: String,
    pub begin_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub begin_ticks: i64,
    pub end_ticks: i64,
    pub nb_objects: i32,
}

/// The `BlockPayload` sent along with the `Block`.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockPayload {
    pub dependencies: Vec<u8>,
    pub objects: Vec<u8>,
}

/// The `BlockMedatada` saved in the database.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockMetadata {
    pub block_id: String,
    pub stream_id: String,
    pub begin_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub begin_ticks: i64,
    pub end_ticks: i64,
    pub nb_objects: i32,
    pub payload_size: i64,
}

impl From<BlockPayload> for lgn_telemetry_proto::telemetry::BlockPayload {
    fn from(payload: BlockPayload) -> Self {
        Self {
            dependencies: payload.dependencies,
            objects: payload.objects,
        }
    }
}

/// Encode a block information and its payload into a buffer.
///
/// # Errors
///
/// This function will return an error if encoding fails.
pub fn encode_block_and_payload(block: &Block, payload: BlockPayload) -> Result<Vec<u8>> {
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
    let block: Block = serde_json::from_str(&block_text)?;

    let dependencies = read_binary_chunk(data, &mut offset)?;
    let objects = read_binary_chunk(data, &mut offset)?;
    let payload = BlockPayload {
        dependencies,
        objects,
    };

    if offset != data.len() {
        warn!("decode_block_and_payload: data was not parsed completely");
    }

    Ok((block, payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let block = Block {
            block_id: "123".to_string(),
            stream_id: "456".to_string(),
            begin_time: chrono::DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc),
            end_time: chrono::DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc),
            begin_ticks: 1,
            end_ticks: 2,
            nb_objects: 3,
        };
        let payload = BlockPayload {
            dependencies: b"123".to_vec(),
            objects: b"456".to_vec(),
        };
        let data = encode_block_and_payload(&block, payload.clone()).unwrap();

        let (block2, payload2) = decode_block_and_payload(&data).unwrap();
        assert_eq!(block, block2);
        assert_eq!(payload, payload2);
    }
}
