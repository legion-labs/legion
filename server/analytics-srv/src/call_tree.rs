use std::path::Path;

use anyhow::Result;
use legion_analytics::prelude::*;
use legion_telemetry_proto::analytics::ScopeInstance;
use legion_transit::prelude::*;

trait ThreadBlockProcessor {
    fn on_begin_scope(&mut self, scope_name: String, ts: u64);
    fn on_end_scope(&mut self, scope_name: String, ts: u64);
}

async fn parse_thread_bock<Proc: ThreadBlockProcessor>(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    stream: &legion_telemetry::StreamInfo,
    block_id: &str,
    processor: &mut Proc,
) -> Result<()> {
    let payload = fetch_block_payload(connection, data_path, block_id).await?;
    parse_block(stream, &payload, |val| {
        if let Value::Object(obj) = val {
            let tick = obj.get::<u64>("time").unwrap();
            let scope = obj.get::<Object>("scope").unwrap();
            let name = scope.get::<String>("name").unwrap();
            match obj.type_name.as_str() {
                "BeginScopeEvent" => processor.on_begin_scope(name, tick),
                "EndScopeEvent" => processor.on_end_scope(name, tick),
                _ => panic!("unknown event type {}", obj.type_name),
            };
        }
        true //continue
    })?;
    Ok(())
}

struct CallTreeBuilder {
    ts_begin_block: u64,
    ts_end_block: u64,
    ts_offset: u64,
    inv_tsc_frequency: f64,
    stack: Vec<ScopeInstance>,
}

impl CallTreeBuilder {
    pub fn new(
        ts_begin_block: u64,
        ts_end_block: u64,
        ts_offset: u64,
        inv_tsc_frequency: f64,
    ) -> Self {
        Self {
            ts_begin_block,
            ts_end_block,
            ts_offset,
            inv_tsc_frequency,
            stack: Vec::new(),
        }
    }

    pub fn finish(mut self) -> ScopeInstance {
        if self.stack.is_empty() {
            return ScopeInstance {
                name: String::new(),
                begin_ms: self.get_time(self.ts_begin_block),
                end_ms: self.get_time(self.ts_end_block),
                scopes: vec![],
            };
        }
        while !self.stack.len() > 1 {
            let top = self.stack.pop().unwrap();
            let last_index = self.stack.len() - 1;
            let parent = &mut self.stack[last_index];
            parent.scopes.push(top);
        }
        assert_eq!(1, self.stack.len());
        self.stack.pop().unwrap()
    }

    #[allow(clippy::cast_precision_loss)]
    fn get_time(&self, ts: u64) -> f64 {
        (ts - self.ts_offset) as f64 * self.inv_tsc_frequency
    }

    fn add_child_to_top(&mut self, scope: ScopeInstance) {
        if let Some(mut top) = self.stack.pop() {
            top.scopes.push(scope);
            self.stack.push(top);
        } else {
            let new_root = ScopeInstance {
                name: String::new(),
                begin_ms: self.get_time(self.ts_begin_block),
                end_ms: self.get_time(self.ts_end_block),
                scopes: vec![scope],
            };
            self.stack.push(new_root);
        }
    }
}

impl ThreadBlockProcessor for CallTreeBuilder {
    fn on_begin_scope(&mut self, scope_name: String, ts: u64) {
        let time = self.get_time(ts);
        let scope = ScopeInstance {
            name: scope_name,
            begin_ms: time,
            end_ms: self.get_time(self.ts_end_block),
            scopes: Vec::new(),
        };
        self.stack.push(scope);
    }

    fn on_end_scope(&mut self, scope_name: String, ts: u64) {
        let time = self.get_time(ts);
        if let Some(mut old_top) = self.stack.pop() {
            if old_top.name == scope_name {
                old_top.end_ms = time;
                self.add_child_to_top(old_top);
            } else if old_top.name.is_empty() {
                old_top.name = scope_name;
                old_top.end_ms = time;
                self.add_child_to_top(old_top);
            } else {
                panic!("top scope mismatch");
            }
        } else {
            let scope = ScopeInstance {
                name: scope_name,
                begin_ms: self.get_time(self.ts_begin_block),
                end_ms: time,
                scopes: Vec::new(),
            };
            self.add_child_to_top(scope);
        }
    }
}

#[allow(clippy::cast_precision_loss)]
pub(crate) async fn compute_block_call_tree(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    process: &legion_telemetry::ProcessInfo,
    stream: &legion_telemetry::StreamInfo,
    block_id: &str,
) -> Result<ScopeInstance> {
    let ts_offset = process.start_ticks;
    let inv_tsc_frequency = 1000.0 / process.tsc_frequency as f64;
    let block = find_block(connection, block_id).await?;
    let mut builder = CallTreeBuilder::new(
        block.begin_ticks,
        block.end_ticks,
        ts_offset,
        inv_tsc_frequency,
    );
    parse_thread_bock(connection, data_path, stream, block_id, &mut builder).await?;
    Ok(builder.finish())
}
