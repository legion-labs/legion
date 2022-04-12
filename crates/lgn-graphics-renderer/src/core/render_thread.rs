use crossbeam_channel::{bounded, Receiver, Sender};
use lgn_tasks::{ComputeTaskPool, Scope, TaskPoolBuilder};
use lgn_tracing::{async_span_scope, span_fn, span_scope};

pub enum RenderThreadCommand {
    RenderFrame(u64),
    Shutdown,
}

pub enum RenderThreadResult {
    RenderedFrame(u64),
}

pub(crate) struct RenderThreadContext {
    render_task_pool: ComputeTaskPool,
    command_receiver: Receiver<RenderThreadCommand>,
    result_sender: Sender<RenderThreadResult>,
}

impl RenderThreadContext {
    pub(crate) fn new(
        command_receiver: Receiver<RenderThreadCommand>,
        result_sender: Sender<RenderThreadResult>,
    ) -> Self {
        Self {
            render_task_pool: ComputeTaskPool(
                TaskPoolBuilder::default()
                    // TODO(ader) - use a real thread count
                    .num_threads(6)
                    .thread_name("Render Task Pool".to_string())
                    .build(),
            ),
            command_receiver,
            result_sender,
        }
    }
}

pub(crate) struct RenderThread {
    _thread: std::thread::JoinHandle<()>,
    command_sender: Sender<RenderThreadCommand>,
    result_receiver: Receiver<RenderThreadResult>,
    frame_count: u64,
}

impl RenderThread {
    pub(crate) fn new() -> Self {
        let (command_sender, command_receiver) = bounded(2);
        let (result_sender, result_receiver) = bounded(1);
        Self {
            _thread: std::thread::spawn(move || {
                Self::render_loop(command_receiver, result_sender);
            }),
            command_sender,
            result_receiver,
            frame_count: 0,
        }
    }

    #[span_fn]
    pub(crate) fn kickoff_render_frame(&mut self) {
        self.command_sender
            .send(RenderThreadCommand::RenderFrame(self.frame_count))
            .expect("Render command channel disconnected!");
        self.frame_count += 1;
    }

    #[span_fn]
    pub(crate) fn wait_for_previous_render_frame(&mut self) {
        if self.frame_count > 0 {
            let result = self
                .result_receiver
                .recv()
                .expect("Render result channel disconnected!");

            match result {
                RenderThreadResult::RenderedFrame(frame_num) => {
                    assert_eq!(frame_num, self.frame_count - 1);
                }
            };
        }
    }

    fn render_loop(
        command_receiver: Receiver<RenderThreadCommand>,
        result_sender: Sender<RenderThreadResult>,
    ) {
        let render_context = RenderThreadContext::new(command_receiver, result_sender);

        for command in render_context.command_receiver.iter() {
            match command {
                RenderThreadCommand::RenderFrame(frame_num) => {
                    span_scope!("render_therad");

                    render_context
                        .render_task_pool
                        .scope(|scope: &mut Scope<'_, ()>| {
                            for _ in 0..6 {
                                scope.spawn(async move {
                                    async_span_scope!("fake_render_task");

                                    let start_time = std::time::Instant::now();
                                    let duration = std::time::Duration::from_secs_f32(0.005);

                                    while std::time::Instant::now() - start_time < duration {
                                        // Spinning for 'duration', simulating doing hard
                                        // compute work generating translation coords!
                                    }
                                });
                            }
                        });
                    render_context
                        .result_sender
                        .send(RenderThreadResult::RenderedFrame(frame_num))
                        .expect("Render thread result sender disconneted!");
                }
                RenderThreadCommand::Shutdown => break,
            }
        }
    }
}
