//! The Presenter's Window module is not a generic Windowing solution,
//! it an opinionated wrapper around the native windowing solution
//!

// todo we should serialize this
/// Location of a window to be created in physical coordinates (without DPI scaling)
pub struct WindowLocation<A: WindowApi> {
    /// monitor index
    pub monitor: A::Monitor,
    /// x coordinate in pixels
    pub x: u32,
    /// y coordinate in pixels
    pub y: u32,
    /// width in pixels
    pub width: u32,
    /// height in pixels
    pub height: u32,
}

/// Window Mode
pub enum WindowMode<A: WindowApi> {
    /// Windowed, fixed size with close, minimize buttons
    Windowed(WindowLocation<A>),
    /// Full screen Borderless
    Borderless(WindowLocation<A>),
    /// Full screen
    Fullscreen(A::Monitor),
}

/// Legion support two window types
pub enum WindowType<A: WindowApi> {
    /// Main window where the application has one window
    Main(WindowMode<A>),

    /// Child window where
    Child(A::WindowHandle),
}

/// Windows creation Api
pub trait WindowApi: Sized {
    /// Native window Handle
    type WindowHandle: WindowHandle<Self>;

    /// Native monitor definition
    type Monitor: Monitor<Self>;

    /// list the monitors
    fn list_monitors() -> Vec<Self::Monitor>;

    /// Create a new Window
    fn new(win_type: WindowType<Self>) -> Self;

    /// get the native window hanlde
    fn native_handle(&self) -> Self::WindowHandle;

    /// blocks to run the event loop
    fn event_loop(&self);
}

/// opaque native handle
pub trait WindowHandle<A: WindowApi>: raw_window_handle::HasRawWindowHandle {}

/// monitor info
pub trait Monitor<A: WindowApi>: Copy + Clone {
    /// Monitor size on physical pixels
    fn size(&self) -> (u32, u32);

    /// Monitor Scale Factor
    fn scale_factor(&self) -> f32;
}

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsWindow as Window;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxWindow as Window;
