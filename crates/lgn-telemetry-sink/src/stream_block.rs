use anyhow::Result;
use lgn_telemetry::{
    compress,
    types::{Block, BlockPayload},
};
use lgn_tracing::{
    event::{ExtractDeps, TracingBlock},
    logs::LogBlock,
    metrics::MetricsBlock,
    spans::ThreadBlock,
};

pub trait StreamBlock {
    fn encode(&self) -> Result<(Block, BlockPayload)>;
}

impl StreamBlock for LogBlock {
    #[allow(clippy::cast_possible_wrap)]
    fn encode(&self) -> Result<(Block, BlockPayload)> {
        let block_id = uuid::Uuid::new_v4().to_string();
        let end = self.end.as_ref().unwrap();

        let payload = BlockPayload {
            dependencies: compress(self.events.extract().as_bytes())?,
            objects: compress(self.events.as_bytes())?,
        };

        Ok((
            Block {
                stream_id: self.stream_id.clone(),
                block_id,
                begin_time: self.begin.time,
                begin_ticks: self.begin.ticks,
                end_time: end.time,
                end_ticks: end.ticks,
                nb_objects: self.nb_objects() as i32,
            },
            payload,
        ))
    }
}

impl StreamBlock for MetricsBlock {
    #[allow(clippy::cast_possible_wrap)]
    fn encode(&self) -> Result<(Block, BlockPayload)> {
        let block_id = uuid::Uuid::new_v4().to_string();
        let end = self.end.as_ref().unwrap();

        let payload = BlockPayload {
            dependencies: compress(self.events.extract().as_bytes())?,
            objects: compress(self.events.as_bytes())?,
        };

        Ok((
            Block {
                stream_id: self.stream_id.clone(),
                block_id,
                begin_time: self.begin.time,
                begin_ticks: self.begin.ticks,
                end_time: end.time,
                end_ticks: end.ticks,
                nb_objects: self.nb_objects() as i32,
            },
            payload,
        ))
    }
}

impl StreamBlock for ThreadBlock {
    #[allow(clippy::cast_possible_wrap)]
    fn encode(&self) -> Result<(Block, BlockPayload)> {
        let block_id = uuid::Uuid::new_v4().to_string();
        let end = self.end.as_ref().unwrap();

        let payload = BlockPayload {
            dependencies: compress(self.events.extract().as_bytes())?,
            objects: compress(self.events.as_bytes())?,
        };

        Ok((
            Block {
                stream_id: self.stream_id.clone(),
                block_id,
                begin_time: self.begin.time,
                begin_ticks: self.begin.ticks,
                end_time: end.time,
                end_ticks: end.ticks,
                nb_objects: self.nb_objects() as i32,
            },
            payload,
        ))
    }
}
