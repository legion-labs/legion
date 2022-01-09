use std::collections::HashSet;

use lgn_tracing_transit::prelude::*;

use crate::{
    event_block::{EventBlock, ExtractDeps},
    event_stream::EventStream,
    log_events::{
        LogDesc, LogStaticStrEvent, LogStaticStrInteropEvent, LogStringEvent,
        LogStringInteropEvent, ReferencedLogDesc,
    },
};

declare_queue_struct!(
    struct LogMsgQueue<
        LogStaticStrEvent,
        LogStringEvent,
        LogStaticStrInteropEvent,
        LogStringInteropEvent,
    > {}
);

declare_queue_struct!(
    struct LogDepsQueue<StaticString, ReferencedLogDesc> {}
);

fn record_log_event_dependencies(
    log_desc: &LogDesc,
    recorded_deps: &mut HashSet<u64>,
    deps: &mut LogDepsQueue,
) {
    let log_ptr = log_desc as *const _ as u64;
    if recorded_deps.insert(log_ptr) {
        let name = StaticString::from(log_desc.fmt_str);
        if recorded_deps.insert(name.ptr as u64) {
            deps.push(name);
        }
        let target = StaticString::from(log_desc.target);
        if recorded_deps.insert(target.ptr as u64) {
            deps.push(target);
        }
        let module_path = StaticString::from(log_desc.module_path);
        if recorded_deps.insert(module_path.ptr as u64) {
            deps.push(module_path);
        }
        let file = StaticString::from(log_desc.file);
        if recorded_deps.insert(file.ptr as u64) {
            deps.push(file);
        }
        deps.push(ReferencedLogDesc {
            id: log_ptr,
            level: log_desc.level,
            fmt_str: log_desc.fmt_str.as_ptr(),
            target: log_desc.target.as_ptr(),
            module_path: log_desc.module_path.as_ptr(),
            file: log_desc.file.as_ptr(),
            line: log_desc.line,
        });
    }
}

impl ExtractDeps for LogMsgQueue {
    type DepsQueue = LogDepsQueue;

    fn extract(&self) -> Self::DepsQueue {
        let mut deps = LogDepsQueue::new(1024 * 1024);
        let mut recorded_deps = HashSet::new();
        for x in self.iter() {
            match x {
                LogMsgQueueAny::LogStaticStrEvent(evt) => {
                    record_log_event_dependencies(evt.desc, &mut recorded_deps, &mut deps);
                }
                LogMsgQueueAny::LogStringEvent(evt) => {
                    record_log_event_dependencies(evt.desc, &mut recorded_deps, &mut deps);
                }
                LogMsgQueueAny::LogStaticStrInteropEvent(evt) => {
                    if recorded_deps.insert(evt.msg as u64) {
                        deps.push(StaticString {
                            len: evt.msg_len,
                            ptr: evt.msg,
                        });
                    }
                }
                LogMsgQueueAny::LogStringInteropEvent(_evt) => {}
            }
        }
        deps
    }
}

pub type LogBlock = EventBlock<LogMsgQueue>;
pub type LogStream = EventStream<LogBlock>;
