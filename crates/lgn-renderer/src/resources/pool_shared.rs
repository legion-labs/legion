pub(crate) trait OnFrameEventHandler {
    fn on_begin_frame(&mut self);
    fn on_end_frame(&mut self);
}
