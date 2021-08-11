#![allow(unsafe_code)]

use raw_window_handle::{windows, HasRawWindowHandle, RawWindowHandle};
use widestring::U16CString;
use winapi::{
    shared::{minwindef, windef},
    um::{libloaderapi, shellscalingapi, winuser},
};

use crate::window;

use super::{Monitor, WindowApi, WindowHandle};

pub struct WindowsWindowHandle {
    hwnd: windef::HWND,
    hinstance: minwindef::HINSTANCE,
}

#[derive(Clone, Copy)]
pub struct WindowsMonitor {
    _hmonitor: windef::HMONITOR,
    width: u32,
    height: u32,
    scale_factor: f32,
}

/// Windows window definition
pub struct WindowsWindow {
    hwnd: windef::HWND,
}

pub const BASE_DPI: f32 = 96.0;

impl WindowApi for WindowsWindow {
    type WindowHandle = WindowsWindowHandle;
    type Monitor = WindowsMonitor;

    fn list_monitors() -> Vec<WindowsMonitor> {
        unsafe extern "system" fn monitor_enum_proc(
            hmonitor: windef::HMONITOR,
            _hdc: windef::HDC,
            _place: windef::LPRECT,
            data: minwindef::LPARAM,
        ) -> minwindef::BOOL {
            let monitors = data as *mut Vec<WindowsMonitor>;

            if let Some((width, height)) = monitor_info(hmonitor) {
                (*monitors).push(WindowsMonitor {
                    _hmonitor: hmonitor,
                    width,
                    height,
                    scale_factor: if let Some(dpi) = monitor_dpi(hmonitor) {
                        dpi as f32 / BASE_DPI
                    } else {
                        1.0
                    },
                });
            }
            minwindef::TRUE
        }

        let mut monitors: Vec<WindowsMonitor> = Vec::new();
        unsafe {
            winuser::EnumDisplayMonitors(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                Some(monitor_enum_proc),
                &mut monitors as *mut _ as minwindef::LPARAM,
            );
        }
        monitors
    }

    // Create Centered Window
    //RECT rect = { 0, 0, popNumericCast<LONG>(wantedWidth), popNumericCast<LONG>(wantedHeight) };
    //winuser::AdjustWindowRect(&rect, MainWindowStyle, FALSE);

    fn new(win_type: super::WindowType<Self>) -> Self {
        let (style, x, y, width, height, parent) = match win_type {
            crate::window::WindowType::Main(window_mode) => match window_mode {
                window::WindowMode::Windowed(location) => {
                    let style = winuser::WS_OVERLAPPED
                        | winuser::WS_CAPTION
                        | winuser::WS_SYSMENU
                        | winuser::WS_MINIMIZEBOX
                        | winuser::WS_CLIPSIBLINGS
                        | winuser::WS_CLIPCHILDREN
                        | winuser::WS_VISIBLE;
                    let mut rect = windef::RECT {
                        left: 0,
                        top: 0,
                        right: location.width as i32,
                        bottom: location.height as i32,
                    };
                    println!("{:?}", rect);
                    unsafe {
                        winuser::AdjustWindowRect(&mut rect, style, minwindef::FALSE);
                    }
                    println!("{:?}", rect);
                    (
                        style,
                        location.x as i32,
                        location.y as i32,
                        rect.right - rect.left,
                        rect.bottom - rect.top,
                        std::ptr::null_mut::<windef::HWND__>(),
                    )
                }
                window::WindowMode::Borderless(location) => (
                    winuser::WS_POPUP | winuser::WS_CLIPSIBLINGS | winuser::WS_CLIPCHILDREN,
                    location.x as i32,
                    location.y as i32,
                    location.width as i32,
                    location.height as i32,
                    std::ptr::null_mut::<windef::HWND__>(),
                ),
                window::WindowMode::Fullscreen(monitor) => (
                    winuser::WS_POPUP | winuser::WS_CLIPSIBLINGS | winuser::WS_CLIPCHILDREN,
                    0,
                    0,
                    monitor.size().0 as i32,
                    monitor.size().1 as i32,
                    std::ptr::null_mut::<windef::HWND__>(),
                ),
            },
            crate::window::WindowType::Child(handle) => (
                winuser::WS_CHILD | winuser::WS_CLIPSIBLINGS | winuser::WS_CLIPCHILDREN,
                winuser::CW_USEDEFAULT,
                winuser::CW_USEDEFAULT,
                winuser::CW_USEDEFAULT,
                winuser::CW_USEDEFAULT,
                handle.hwnd,
            ),
        };
        let hwnd = unsafe {
            winapi::um::winuser::CreateWindowExW(
                0,
                WINDOWS_GLOBALS.class_name.as_ptr(),
                WINDOWS_GLOBALS.class_name.as_ptr(),
                style,
                x,
                y,
                width,
                height,
                parent,
                std::ptr::null_mut(),
                WINDOWS_GLOBALS.hinstance,
                std::ptr::null_mut(),
            )
        };
        if hwnd.is_null() {
            panic!("failed to create a window");
        }
        Self { hwnd }
    }

    fn native_handle(&self) -> WindowsWindowHandle {
        WindowsWindowHandle {
            hwnd: self.hwnd,
            hinstance: WINDOWS_GLOBALS.hinstance,
        }
    }

    fn event_loop(&self) {
        unsafe { winuser::ShowWindow(self.hwnd, winuser::SW_SHOW) };
        let mut msg = winuser::MSG {
            hwnd: std::ptr::null_mut(),
            message: 0,
            wParam: 0,
            lParam: 0,
            time: 0,
            pt: windef::POINT { x: 0, y: 0 },
        };
        unsafe {
            while winuser::GetMessageW(&mut msg, ::std::ptr::null_mut(), 0, 0) != 0 {
                winuser::TranslateMessage(&msg);
                winuser::DispatchMessageW(&msg);
            }
        };
    }
}

#[allow(unsafe_code)]
unsafe impl HasRawWindowHandle for WindowsWindowHandle {
    fn raw_window_handle(&self) -> RawWindowHandle {
        RawWindowHandle::Windows(windows::WindowsHandle {
            hwnd: self.hwnd.cast(),
            hinstance: self.hinstance.cast(),
            ..windows::WindowsHandle::empty()
        })
    }
}

impl WindowHandle<WindowsWindow> for WindowsWindowHandle {}

impl Monitor<WindowsWindow> for WindowsMonitor {
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn scale_factor(&self) -> f32 {
        self.scale_factor
    }
}

pub fn monitor_dpi(hmonitor: windef::HMONITOR) -> Option<u32> {
    let mut dpi_x = 0;
    let mut dpi_y = 0;
    unsafe {
        if shellscalingapi::GetDpiForMonitor(
            hmonitor,
            shellscalingapi::MDT_EFFECTIVE_DPI,
            &mut dpi_x,
            &mut dpi_y,
        ) == 0
        {
            return Some(dpi_x as u32);
        }
    }
    None
}

fn monitor_info(hmonitor: windef::HMONITOR) -> Option<(u32, u32)> {
    let mut monitor_info: winuser::MONITORINFOEXW = unsafe { std::mem::zeroed() };
    monitor_info.cbSize = std::mem::size_of::<winuser::MONITORINFOEXW>() as minwindef::DWORD;
    let status = unsafe {
        winuser::GetMonitorInfoW(
            hmonitor,
            (&mut monitor_info as *mut winuser::MONITORINFOEXW).cast(),
        )
    };
    if status != 0 {
        Some((
            (monitor_info.rcMonitor.right - monitor_info.rcMonitor.left) as _,
            (monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top) as _,
        ))
    } else {
        None
    }
}

extern "system" fn window_proc(
    hwnd: windef::HWND,
    message: minwindef::UINT,
    wparam: minwindef::WPARAM,
    lparam: minwindef::LPARAM,
) -> minwindef::LRESULT {
    unsafe { winapi::um::winuser::DefWindowProcW(hwnd, message, wparam, lparam) }
}

struct WindowsGlobals {
    hinstance: minwindef::HINSTANCE,
    class_name: U16CString,
}

// HINSTANCE is safe to share globally
unsafe impl Send for WindowsGlobals {}
unsafe impl Sync for WindowsGlobals {}

fn create_window_class() -> WindowsGlobals {
    // this should never fail
    let class_name = U16CString::from_str("Legion").unwrap();

    let hinstance = unsafe { libloaderapi::GetModuleHandleW(std::ptr::null()) };

    let wnd_class = winapi::um::winuser::WNDCLASSEXW {
        cbSize: ::std::mem::size_of::<winapi::um::winuser::WNDCLASSEXW>() as u32,
        style: winapi::um::winuser::CS_HREDRAW | winapi::um::winuser::CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: std::ptr::null_mut::<windef::HICON__>(),
        hCursor: std::ptr::null_mut::<windef::HICON__>(),
        hbrBackground: std::ptr::null_mut::<windef::HBRUSH__>(),
        lpszMenuName: std::ptr::null::<u16>(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: std::ptr::null_mut::<windef::HICON__>(),
    };
    unsafe {
        winapi::um::winuser::RegisterClassExW(std::ptr::addr_of!(wnd_class));
    }
    WindowsGlobals {
        hinstance,
        class_name,
    }
}

lazy_static::lazy_static! {
    static ref WINDOWS_GLOBALS: WindowsGlobals = create_window_class();
}

/*
pub fn parent(&self, parent: winapi::shared::windef::HWND) {
        unsafe {
            let parent_style =
                winapi::um::winuser::GetWindowLongPtrW(parent, winapi::um::winuser::GWL_STYLE);
            //assert!(parent_style & winapi::um::winuser::WS_CLIPSIBLINGS as isize == 0);
            winapi::um::winuser::SetWindowLongPtrW(
                parent,
                winapi::um::winuser::GWL_STYLE,
                parent_style
                    | (winapi::um::winuser::WS_CLIPSIBLINGS | winapi::um::winuser::WS_CLIPCHILDREN)
                        as isize,
            );
            winapi::um::winuser::SetWindowLongPtrW(
                self.hwnd,
                winapi::um::winuser::GWL_STYLE,
                (winapi::um::winuser::WS_CHILD
                    | winapi::um::winuser::WS_CLIPSIBLINGS
                    | winapi::um::winuser::WS_CLIPCHILDREN
                    | winapi::um::winuser::WS_VISIBLE) as isize,
            );
            let hwnd = winapi::um::winuser::SetParent(self.hwnd, parent);
            if hwnd == std::ptr::null_mut() {
                let error = winapi::um::errhandlingapi::GetLastError();
                println!("Set parent error: {:?}", error);
            }
            //winapi::um::winuser::SetWindowLongPtrW(
            //    self.hwnd,
            //    winapi::um::winuser::GWL_EXSTYLE,
            //    winapi::um::winuser::WS_EX_TOPMOST as isize,
            //);

            winapi::um::winuser::SetWindowPos(
                self.hwnd,
                winapi::um::winuser::HWND_TOPMOST,
                winapi::um::winuser::CW_USEDEFAULT,
                winapi::um::winuser::CW_USEDEFAULT,
                winapi::um::winuser::CW_USEDEFAULT,
                winapi::um::winuser::CW_USEDEFAULT,
                0,
            );
        };
    }
*/
