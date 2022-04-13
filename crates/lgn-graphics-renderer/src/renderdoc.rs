use std::sync::Mutex;

use lgn_app::{App, CoreStage, EventReader, Plugin};
use lgn_ecs::prelude::ResMut;
use lgn_input::keyboard::{KeyCode, KeyboardInput};
use lgn_tracing::{info, warn};
use renderdoc::{RenderDoc, Version};

pub struct RenderDocPlugin {}

type RenderDocVersion = renderdoc::V141;

impl Plugin for RenderDocPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderDocManager>();
        app.add_system_to_stage(CoreStage::First, start_frame_capture);
        app.add_system_to_stage(CoreStage::Last, end_frame_capture);
        app.add_system(listen_for_key);
    }
}

#[derive(PartialEq)]
enum CaptureState {
    NotAvailable,
    NotCapturing,
    CaptureScheduled,
    CaptureStarted,
}
struct RenderDocManager {
    rd: Option<Mutex<RenderDoc<RenderDocVersion>>>,
    capture_state: CaptureState,
}

#[allow(unsafe_code)]
unsafe impl Send for RenderDocManager {}
#[allow(unsafe_code)]
unsafe impl Sync for RenderDocManager {}

impl RenderDocManager {
    pub fn schedule_capture(&mut self) {
        if self.capture_state == CaptureState::NotAvailable {
            warn!("Render Doc is not available. Make sure you are starting from Render Doc and the version is at least {:?}", RenderDocVersion::VERSION);
        } else if self.capture_state == CaptureState::NotCapturing {
            info!("RenderDoc capture scheduled");
            self.capture_state = CaptureState::CaptureScheduled;
        }
    }

    pub fn start_frame_capture(&mut self) {
        if self.capture_state == CaptureState::CaptureScheduled {
            info!("RenderDoc capture started");
            self.rd
                .as_ref()
                .unwrap()
                .lock()
                .unwrap()
                .start_frame_capture(std::ptr::null(), std::ptr::null());
            self.capture_state = CaptureState::CaptureStarted;
        }
    }

    pub fn end_frame_capture(&mut self) {
        if self.capture_state == CaptureState::CaptureStarted {
            info!("RenderDoc capture ended");
            self.rd
                .as_ref()
                .unwrap()
                .lock()
                .unwrap()
                .end_frame_capture(std::ptr::null(), std::ptr::null());
            self.capture_state = CaptureState::NotCapturing;
        }
    }
}

impl Default for RenderDocManager {
    fn default() -> Self {
        let rd = RenderDoc::new();
        if let Ok(rd) = rd {
            return Self {
                rd: Some(Mutex::new(rd)),
                capture_state: CaptureState::NotCapturing,
            };
        }

        Self {
            rd: None,
            capture_state: CaptureState::NotAvailable,
        }
    }
}

fn listen_for_key(
    mut renderdoc: ResMut<'_, RenderDocManager>,
    mut keyboard_input_events: EventReader<'_, '_, KeyboardInput>,
) {
    for keyboard_input_event in keyboard_input_events.iter() {
        if let Some(key_code) = keyboard_input_event.key_code {
            if key_code == KeyCode::C && keyboard_input_event.state.is_pressed() {
                renderdoc.schedule_capture();
            }
        }
    }
}

fn start_frame_capture(mut renderdoc: ResMut<'_, RenderDocManager>) {
    renderdoc.start_frame_capture();
}

fn end_frame_capture(mut renderdoc: ResMut<'_, RenderDocManager>) {
    renderdoc.end_frame_capture();
}
