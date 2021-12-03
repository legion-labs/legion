//
// TODO: At some point, it might be better to add a FrameEvent trait with on_begin_frame, on_end_frame???
//
pub(crate) trait OnNewFrame {
    fn on_new_frame(&mut self);
}
