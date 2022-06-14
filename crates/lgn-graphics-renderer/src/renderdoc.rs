use lgn_tracing::{info, warn};

use renderdoc::Version;

use crate::core::{RenderCommand, RenderResources};

type RenderDocVersion = renderdoc::V141;

#[derive(Clone, Copy, PartialEq)]
enum CaptureState {
    NotCapturing,
    CaptureScheduled,
    CaptureStarted,
}
pub(crate) struct RenderDocManager {
    rd: Option<renderdoc::RenderDoc<RenderDocVersion>>,
    capture_state: CaptureState,
}

impl RenderDocManager {
    pub fn start_frame_capture(&mut self) {
        if self.capture_state == CaptureState::CaptureScheduled {
            if let Some(rd) = self.rd.as_mut() {
                info!("RenderDoc capture started");
                rd.start_frame_capture(std::ptr::null(), std::ptr::null());
                self.capture_state = CaptureState::CaptureStarted;
            } else {
                warn!("Render Doc is not available. Make sure you are starting from Render Doc and the version is at least {:?}", RenderDocVersion::VERSION);
            }
        }
    }

    pub fn end_frame_capture(&mut self) {
        if self.capture_state == CaptureState::CaptureStarted {
            if let Some(rd) = self.rd.as_mut() {
                info!("RenderDoc capture ended");
                rd.end_frame_capture(std::ptr::null(), std::ptr::null());
                self.capture_state = CaptureState::NotCapturing;
            }
        }
    }

    fn schedule_capture(&mut self) {
        if self.capture_state == CaptureState::NotCapturing {
            info!("RenderDoc capture scheduled");
            self.capture_state = CaptureState::CaptureScheduled;
        }
    }
}

impl Default for RenderDocManager {
    fn default() -> Self {
        Self {
            rd: renderdoc::RenderDoc::new().ok(),
            capture_state: CaptureState::NotCapturing,
        }
    }
}

#[allow(unsafe_code)]
unsafe impl Sync for RenderDocManager {}

#[derive(Default)]
pub struct RenderDocCaptureCommand;

impl RenderCommand<RenderResources> for RenderDocCaptureCommand {
    fn execute(self, render_resources: &RenderResources) {
        let mut renderdoc_manager = render_resources.get_mut::<RenderDocManager>();
        renderdoc_manager.schedule_capture();
    }
}
