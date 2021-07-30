#![allow(unsafe_code)]

use raw_window_handle::{unix::WaylandHandle, HasRawWindowHandle, RawWindowHandle};

use super::{Monitor, WindowApi, WindowHandle};

/// Linux Window
pub struct LinuxWindow {}

/// Linux Window Handle
pub struct LinuxWindowHandle {}

/// Linux Monitor
#[derive(Clone, Copy)]
pub struct LinuxMonitor {}

impl WindowApi for LinuxWindow {
    type WindowHandle = LinuxWindowHandle;

    type Monitor = LinuxMonitor;

    fn list_monitors() -> Vec<Self::Monitor> {
        vec![]
    }

    fn new(_win_type: super::WindowType<Self>) -> Self {
        Self {}
    }

    fn native_handle(&self) -> Self::WindowHandle {
        LinuxWindowHandle {}
    }

    fn event_loop(&self) {}
}

impl Monitor<LinuxWindow> for LinuxMonitor {
    fn size(&self) -> (u32, u32) {
        (0, 0)
    }

    fn scale_factor(&self) -> f32 {
        1.0
    }
}

impl WindowHandle<LinuxWindow> for LinuxWindowHandle {}

unsafe impl HasRawWindowHandle for LinuxWindowHandle {
    fn raw_window_handle(&self) -> RawWindowHandle {
        RawWindowHandle::Wayland(WaylandHandle::empty())
    }
}
