use legion_math::IVec2;
use legion_utils::HashMap;
use legion_window::{Window, WindowDescriptor, WindowId, WindowMode};
use tao::dpi::LogicalSize;

#[derive(Debug, Default)]
pub struct TaoWindows {
    pub windows: HashMap<tao::window::WindowId, tao::window::Window>,
    pub window_id_to_tao: HashMap<WindowId, tao::window::WindowId>,
    pub tao_to_window_id: HashMap<tao::window::WindowId, WindowId>,
}

impl TaoWindows {
    pub fn create_window(
        &mut self,
        event_loop: &tao::event_loop::EventLoopWindowTarget<()>,
        window_id: WindowId,
        window_descriptor: &WindowDescriptor,
    ) -> Window {
        #[cfg(target_os = "windows")]
        let mut tao_window_builder = {
            use tao::platform::windows::WindowBuilderExtWindows;
            tao::window::WindowBuilder::new().with_drag_and_drop(false)
        };

        #[cfg(not(target_os = "windows"))]
        let mut tao_window_builder = tao::window::WindowBuilder::new();

        tao_window_builder = match window_descriptor.mode {
            WindowMode::BorderlessFullscreen => tao_window_builder.with_fullscreen(Some(
                tao::window::Fullscreen::Borderless(event_loop.primary_monitor()),
            )),
            WindowMode::Fullscreen { use_size } => tao_window_builder.with_fullscreen(Some(
                tao::window::Fullscreen::Exclusive(match use_size {
                    true => get_fitting_videomode(
                        &event_loop.primary_monitor().unwrap(),
                        window_descriptor.width as u32,
                        window_descriptor.height as u32,
                    ),
                    false => get_best_videomode(&event_loop.primary_monitor().unwrap()),
                }),
            )),
            _ => {
                let WindowDescriptor {
                    width,
                    height,
                    scale_factor_override,
                    ..
                } = window_descriptor;
                if let Some(sf) = scale_factor_override {
                    tao_window_builder.with_inner_size(
                        tao::dpi::LogicalSize::new(*width, *height).to_physical::<f64>(*sf),
                    )
                } else {
                    tao_window_builder.with_inner_size(tao::dpi::LogicalSize::new(*width, *height))
                }
            }
            .with_resizable(window_descriptor.resizable)
            .with_decorations(window_descriptor.decorations),
        };

        let constraints = window_descriptor.resize_constraints.check_constraints();
        let min_inner_size = LogicalSize {
            width: constraints.min_width,
            height: constraints.min_height,
        };
        let max_inner_size = LogicalSize {
            width: constraints.max_width,
            height: constraints.max_height,
        };

        let tao_window_builder =
            if constraints.max_width.is_finite() && constraints.max_height.is_finite() {
                tao_window_builder
                    .with_min_inner_size(min_inner_size)
                    .with_max_inner_size(max_inner_size)
            } else {
                tao_window_builder.with_min_inner_size(min_inner_size)
            };

        #[allow(unused_mut)]
        let mut tao_window_builder = tao_window_builder.with_title(&window_descriptor.title);

        #[cfg(target_arch = "wasm32")]
        {
            use tao::platform::web::WindowBuilderExtWebSys;
            use wasm_bindgen::JsCast;

            if let Some(selector) = &window_descriptor.canvas {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let canvas = document
                    .query_selector(&selector)
                    .expect("Cannot query for canvas element.");
                if let Some(canvas) = canvas {
                    let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok();
                    tao_window_builder = tao_window_builder.with_canvas(canvas);
                } else {
                    panic!("Cannot find element: {}.", selector);
                }
            }
        }

        let tao_window = tao_window_builder.build(event_loop).unwrap();

        match tao_window.set_cursor_grab(window_descriptor.cursor_locked) {
            Ok(_) => {}
            Err(tao::error::ExternalError::NotSupported(_)) => {}
            Err(err) => Err(err).unwrap(),
        }

        tao_window.set_cursor_visible(window_descriptor.cursor_visible);

        self.window_id_to_tao.insert(window_id, tao_window.id());
        self.tao_to_window_id.insert(tao_window.id(), window_id);

        #[cfg(target_arch = "wasm32")]
        {
            use tao::platform::web::WindowExtWebSys;

            if window_descriptor.canvas.is_none() {
                let canvas = tao_window.canvas();

                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let body = document.body().unwrap();

                body.append_child(&canvas)
                    .expect("Append canvas to HTML body.");
            }
        }

        let position = tao_window
            .outer_position()
            .ok()
            .map(|position| IVec2::new(position.x, position.y));
        let inner_size = tao_window.inner_size();
        let scale_factor = tao_window.scale_factor();
        self.windows.insert(tao_window.id(), tao_window);
        Window::new(
            window_id,
            window_descriptor,
            inner_size.width,
            inner_size.height,
            scale_factor,
            position,
        )
    }

    pub fn get_window(&self, id: WindowId) -> Option<&tao::window::Window> {
        self.window_id_to_tao
            .get(&id)
            .and_then(|id| self.windows.get(id))
    }

    pub fn get_window_id(&self, id: tao::window::WindowId) -> Option<WindowId> {
        self.tao_to_window_id.get(&id).cloned()
    }
}
pub fn get_fitting_videomode(
    monitor: &tao::monitor::MonitorHandle,
    width: u32,
    height: u32,
) -> tao::monitor::VideoMode {
    let mut modes = monitor.video_modes().collect::<Vec<_>>();

    fn abs_diff(a: u32, b: u32) -> u32 {
        if a > b {
            return a - b;
        }
        b - a
    }

    modes.sort_by(|a, b| {
        use std::cmp::Ordering::*;
        match abs_diff(a.size().width, width).cmp(&abs_diff(b.size().width, width)) {
            Equal => {
                match abs_diff(a.size().height, height).cmp(&abs_diff(b.size().height, height)) {
                    Equal => b.refresh_rate().cmp(&a.refresh_rate()),
                    default => default,
                }
            }
            default => default,
        }
    });

    modes.first().unwrap().clone()
}

pub fn get_best_videomode(monitor: &tao::monitor::MonitorHandle) -> tao::monitor::VideoMode {
    let mut modes = monitor.video_modes().collect::<Vec<_>>();
    modes.sort_by(|a, b| {
        use std::cmp::Ordering::*;
        match b.size().width.cmp(&a.size().width) {
            Equal => match b.size().height.cmp(&a.size().height) {
                Equal => b.refresh_rate().cmp(&a.refresh_rate()),
                default => default,
            },
            default => default,
        }
    });

    modes.first().unwrap().clone()
}

// WARNING: this only works under the assumption that wasm runtime is single threaded
#[cfg(target_arch = "wasm32")]
unsafe impl Send for TaoWindows {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for TaoWindows {}
