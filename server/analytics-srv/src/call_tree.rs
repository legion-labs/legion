use std::path::Path;

use anyhow::Result;
use lgn_analytics::prelude::*;
use lgn_telemetry::prelude::*;
use lgn_telemetry_proto::analytics::BlockSpansReply;
use lgn_telemetry_proto::analytics::ScopeDesc;
use lgn_telemetry_proto::analytics::Span;
use lgn_transit::prelude::*;

trait ThreadBlockProcessor {
    fn on_begin_scope(&mut self, scope_name: String, ts: i64);
    fn on_end_scope(&mut self, scope_name: String, ts: i64);
}

async fn parse_thread_bock<Proc: ThreadBlockProcessor>(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    stream: &lgn_telemetry::StreamInfo,
    block_id: &str,
    processor: &mut Proc,
) -> Result<()> {
    let payload = fetch_block_payload(connection, data_path, block_id).await?;
    parse_block(stream, &payload, |val| {
        trace_scope!("obj_in_block");
        if let Value::Object(obj) = val {
            let tick = obj.get::<i64>("time").unwrap();
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

#[derive(Debug)]
pub(crate) struct CallTreeNode {
    pub hash: u32,
    pub name: String,
    pub scopes: Vec<CallTreeNode>,
    pub begin_ms: f64,
    pub end_ms: f64,
}

struct CallTreeBuilder {
    ts_begin_block: i64,
    ts_end_block: i64,
    ts_offset: i64,
    inv_tsc_frequency: f64,
    stack: Vec<CallTreeNode>,
}

impl CallTreeBuilder {
    pub fn new(
        ts_begin_block: i64,
        ts_end_block: i64,
        ts_offset: i64,
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

    pub fn finish(mut self) -> CallTreeNode {
        trace_scope!();
        if self.stack.is_empty() {
            return CallTreeNode {
                hash: 0,
                name: String::new(),
                begin_ms: self.get_time(self.ts_begin_block),
                end_ms: self.get_time(self.ts_end_block),
                scopes: vec![],
            };
        }
        while self.stack.len() > 1 {
            let top = self.stack.pop().unwrap();
            let last_index = self.stack.len() - 1;
            let parent = &mut self.stack[last_index];
            parent.scopes.push(top);
        }
        assert_eq!(1, self.stack.len());
        self.stack.pop().unwrap()
    }

    #[allow(clippy::cast_precision_loss)]
    fn get_time(&self, ts: i64) -> f64 {
        (ts - self.ts_offset) as f64 * self.inv_tsc_frequency
    }

    fn add_child_to_top(&mut self, scope: CallTreeNode) {
        trace_scope!();
        if let Some(mut top) = self.stack.pop() {
            top.scopes.push(scope);
            self.stack.push(top);
        } else {
            let new_root = CallTreeNode {
                hash: 0,
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
    fn on_begin_scope(&mut self, scope_name: String, ts: i64) {
        trace_scope!();
        let time = self.get_time(ts);
        let scope = CallTreeNode {
            hash: compute_scope_hash(&scope_name),
            name: scope_name,
            begin_ms: time,
            end_ms: self.get_time(self.ts_end_block),
            scopes: Vec::new(),
        };
        self.stack.push(scope);
    }

    fn on_end_scope(&mut self, scope_name: String, ts: i64) {
        trace_scope!();
        let time = self.get_time(ts);
        if let Some(mut old_top) = self.stack.pop() {
            if old_top.name == scope_name {
                old_top.end_ms = time;
                self.add_child_to_top(old_top);
            } else if old_top.name.is_empty() {
                old_top.hash = compute_scope_hash(&scope_name);
                old_top.name = scope_name;
                old_top.end_ms = time;
                self.add_child_to_top(old_top);
            } else {
                panic!("top scope mismatch");
            }
        } else {
            let scope = CallTreeNode {
                hash: compute_scope_hash(&scope_name),
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
    process: &lgn_telemetry::ProcessInfo,
    stream: &lgn_telemetry::StreamInfo,
    block_id: &str,
) -> Result<CallTreeNode> {
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

pub(crate) type ScopeHashMap = std::collections::HashMap<u32, ScopeDesc>;
const CRC32: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISCSI);

fn compute_scope_hash(name: &str) -> u32 {
    //todo: add filename
    CRC32.checksum(name.as_bytes())
}

pub(crate) fn record_scope_in_map(node: &CallTreeNode, scopes: &mut ScopeHashMap) {
    trace_scope!();
    scopes.entry(node.hash).or_insert_with(|| ScopeDesc {
        name: node.name.clone(),
        filename: "".to_string(),
        line: 0,
        hash: node.hash,
    });
}

fn make_spans_from_tree(
    tree: &CallTreeNode,
    depth: u32,
    scopes: &mut ScopeHashMap,
    spans: &mut Vec<Span>,
) -> u32 {
    record_scope_in_map(tree, scopes);
    let span = Span {
        scope_hash: tree.hash,
        depth,
        begin_ms: tree.begin_ms,
        end_ms: tree.end_ms,
    };
    spans.push(span);
    let mut max_depth = depth;
    for child in &tree.scopes {
        max_depth = std::cmp::max(
            max_depth,
            make_spans_from_tree(child, depth + 1, scopes, spans),
        );
    }
    max_depth
}

pub(crate) async fn compute_block_spans(
    connection: &mut sqlx::AnyConnection,
    data_path: &Path,
    process: &lgn_telemetry::ProcessInfo,
    stream: &lgn_telemetry::StreamInfo,
    block_id: &str,
) -> Result<BlockSpansReply> {
    let tree = compute_block_call_tree(connection, data_path, process, stream, block_id).await?;
    let mut scopes = ScopeHashMap::new();
    let mut spans = vec![];
    let mut max_depth = 0;
    if tree.name.is_empty() {
        for child in &tree.scopes {
            max_depth = std::cmp::max(
                max_depth,
                make_spans_from_tree(child, 0, &mut scopes, &mut spans),
            );
        }
    } else {
        max_depth = make_spans_from_tree(&tree, 0, &mut scopes, &mut spans);
    }

    let mut scope_vec = vec![];
    scope_vec.reserve(scopes.len());
    for (_k, v) in scopes.drain() {
        scope_vec.push(v);
    }
    Ok(BlockSpansReply {
        scopes: scope_vec,
        spans,
        block_id: block_id.to_owned(),
        begin_ms: tree.begin_ms,
        end_ms: tree.end_ms,
        max_depth,
    })
}
