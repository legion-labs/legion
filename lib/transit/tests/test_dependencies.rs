use transit::*;

#[derive(Debug)]
pub struct LogMsgEvent {
    pub level: i32,
    pub msg: &'static str,
}

impl Serialize for LogMsgEvent {}

declare_queue_struct!(
    struct LogMsgQueue<LogMsgEvent> {}
);

#[test]
fn test_deps() {
    let mut q = LogMsgQueue::new(1024);
    q.push_log_msg_event(LogMsgEvent {
        level: 0,
        msg: "test_msg",
    });
}
