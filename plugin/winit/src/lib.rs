//! Winit plugin
//!

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow(clippy::too_many_lines)]

mod converters;
mod winit_config;
mod winit_windows;

use lgn_app::{App, AppExit, CoreStage, Events, ManualEventReader, Plugin};
use lgn_ecs::{system::IntoExclusiveSystem, world::World};
use lgn_input::{
    keyboard::KeyboardInput,
    mouse::{MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel},
    touch::TouchInput,
};
use lgn_math::{ivec2, Vec2};
use lgn_window::{
    CreateWindow, CursorEntered, CursorLeft, CursorMoved, FileDragAndDrop, ReceivedCharacter,
    WindowBackendScaleFactorChanged, WindowCloseRequested, WindowCreated, WindowFocused,
    WindowMoved, WindowResized, WindowScaleFactorChanged, Windows,
};
use log::{error, trace, warn};
use winit::dpi::LogicalSize;
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use winit::platform::unix::EventLoopExtUnix;
use winit::{
    dpi::PhysicalPosition,
    event::{self, DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
};
pub use winit_config::*;
pub use winit_windows::*;

#[derive(Default)]
pub struct WinitPlugin;

impl Plugin for WinitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WinitWindows>()
            .set_runner(winit_runner)
            .add_system_to_stage(CoreStage::PostUpdate, change_window.exclusive_system());
    }
}

fn change_window(world: &mut World) {
    let world = world.cell();
    let winit_windows = world.get_resource::<WinitWindows>().unwrap();
    let mut windows = world.get_resource_mut::<Windows>().unwrap();

    for legion_window in windows.iter_mut() {
        let id = legion_window.id();
        for command in legion_window.drain_commands() {
            match command {
                lgn_window::WindowCommand::SetWindowMode {
                    mode,
                    resolution: (width, height),
                } => {
                    let window = winit_windows.get_window(id).unwrap();
                    match mode {
                        lgn_window::WindowMode::BorderlessFullscreen => {
                            window
                                .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                        }
                        lgn_window::WindowMode::Fullscreen { use_size } => window.set_fullscreen(
                            Some(winit::window::Fullscreen::Exclusive(if use_size {
                                get_fitting_videomode(
                                    &window.current_monitor().unwrap(),
                                    width,
                                    height,
                                )
                            } else {
                                get_best_videomode(&window.current_monitor().unwrap())
                            })),
                        ),
                        lgn_window::WindowMode::Windowed => window.set_fullscreen(None),
                    }
                }
                lgn_window::WindowCommand::SetTitle { title } => {
                    let window = winit_windows.get_window(id).unwrap();
                    window.set_title(&title);
                }
                lgn_window::WindowCommand::SetScaleFactor { scale_factor } => {
                    let mut window_dpi_changed_events = world
                        .get_resource_mut::<Events<WindowScaleFactorChanged>>()
                        .unwrap();
                    window_dpi_changed_events.send(WindowScaleFactorChanged { id, scale_factor });
                }
                lgn_window::WindowCommand::SetResolution {
                    logical_resolution: (width, height),
                    scale_factor,
                } => {
                    let window = winit_windows.get_window(id).unwrap();
                    window.set_inner_size(
                        winit::dpi::LogicalSize::new(width, height)
                            .to_physical::<f64>(scale_factor),
                    );
                }
                lgn_window::WindowCommand::SetVsync { .. } => (),
                lgn_window::WindowCommand::SetResizable { resizable } => {
                    let window = winit_windows.get_window(id).unwrap();
                    window.set_resizable(resizable);
                }
                lgn_window::WindowCommand::SetDecorations { decorations } => {
                    let window = winit_windows.get_window(id).unwrap();
                    window.set_decorations(decorations);
                }
                lgn_window::WindowCommand::SetCursorLockMode { locked } => {
                    let window = winit_windows.get_window(id).unwrap();
                    window
                        .set_cursor_grab(locked)
                        .unwrap_or_else(|e| error!("Unable to un/grab cursor: {}", e));
                }
                lgn_window::WindowCommand::SetCursorVisibility { visible } => {
                    let window = winit_windows.get_window(id).unwrap();
                    window.set_cursor_visible(visible);
                }
                lgn_window::WindowCommand::SetCursorPosition { position } => {
                    let window = winit_windows.get_window(id).unwrap();
                    let inner_size = window.inner_size().to_logical::<f32>(window.scale_factor());
                    window
                        .set_cursor_position(winit::dpi::LogicalPosition::new(
                            position.x,
                            inner_size.height - position.y,
                        ))
                        .unwrap_or_else(|e| error!("Unable to set cursor position: {}", e));
                }
                lgn_window::WindowCommand::SetMaximized { maximized } => {
                    let window = winit_windows.get_window(id).unwrap();
                    window.set_maximized(maximized);
                }
                lgn_window::WindowCommand::SetMinimized { minimized } => {
                    let window = winit_windows.get_window(id).unwrap();
                    window.set_minimized(minimized);
                }
                lgn_window::WindowCommand::SetPosition { position } => {
                    let window = winit_windows.get_window(id).unwrap();
                    window.set_outer_position(PhysicalPosition {
                        x: position[0],
                        y: position[1],
                    });
                }
                lgn_window::WindowCommand::SetResizeConstraints { resize_constraints } => {
                    let window = winit_windows.get_window(id).unwrap();
                    let constraints = resize_constraints.check_constraints();
                    let min_inner_size = LogicalSize {
                        width: constraints.min_width,
                        height: constraints.min_height,
                    };
                    let max_inner_size = LogicalSize {
                        width: constraints.max_width,
                        height: constraints.max_height,
                    };

                    window.set_min_inner_size(Some(min_inner_size));
                    if constraints.max_width.is_finite() && constraints.max_height.is_finite() {
                        window.set_max_inner_size(Some(max_inner_size));
                    }
                }
            }
        }
    }
}

fn run<F>(event_loop: EventLoop<()>, event_handler: F) -> !
where
    F: 'static + FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow),
{
    event_loop.run(event_handler)
}

// TODO: It may be worth moving this cfg into a procedural macro so that it can be referenced by
// a single name instead of being copied around.
// https://gist.github.com/jakerr/231dee4a138f7a5f25148ea8f39b382e seems to work.
#[cfg(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
fn run_return<F>(event_loop: &mut EventLoop<()>, event_handler: F)
where
    F: FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow),
{
    use winit::platform::run_return::EventLoopExtRunReturn;
    event_loop.run_return(event_handler);
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
fn run_return<F>(_event_loop: &mut EventLoop<()>, _event_handler: F)
where
    F: FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow),
{
    panic!("Run return is not supported on this platform!")
}

pub fn winit_runner(app: App) {
    winit_runner_with(app, EventLoop::new());
}

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub fn winit_runner_any_thread(app: App) {
    winit_runner_with(app, EventLoop::new_any_thread());
}

pub fn winit_runner_with(mut app: App, mut event_loop: EventLoop<()>) {
    let mut create_window_event_reader = ManualEventReader::<CreateWindow>::default();
    let mut app_exit_event_reader = ManualEventReader::<AppExit>::default();
    app.world.insert_non_send(event_loop.create_proxy());

    trace!("Entering winit event loop");

    let should_return_from_run = app
        .world
        .get_resource::<WinitConfig>()
        .map_or(false, |config| config.return_from_run);

    let mut active = true;

    let event_handler = move |event: Event<'_, ()>,
                              event_loop: &EventLoopWindowTarget<()>,
                              control_flow: &mut ControlFlow| {
        *control_flow = ControlFlow::Poll;

        if let Some(app_exit_events) = app.world.get_resource_mut::<Events<AppExit>>() {
            if app_exit_event_reader
                .iter(&app_exit_events)
                .next_back()
                .is_some()
            {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            event::Event::WindowEvent {
                event,
                window_id: winit_window_id,
                ..
            } => {
                let world = app.world.cell();
                let winit_windows = world.get_resource_mut::<WinitWindows>().unwrap();
                let mut windows = world.get_resource_mut::<Windows>().unwrap();
                let window_id =
                    if let Some(window_id) = winit_windows.get_window_id(winit_window_id) {
                        window_id
                    } else {
                        warn!(
                            "Skipped event for unknown winit Window Id {:?}",
                            winit_window_id
                        );
                        return;
                    };

                let window = if let Some(window) = windows.get_mut(window_id) {
                    window
                } else {
                    warn!("Skipped event for unknown Window Id {:?}", winit_window_id);
                    return;
                };

                match event {
                    WindowEvent::Resized(size) => {
                        window.update_actual_size_from_backend(size.width, size.height);
                        let mut resize_events =
                            world.get_resource_mut::<Events<WindowResized>>().unwrap();
                        resize_events.send(WindowResized {
                            id: window_id,
                            width: window.width(),
                            height: window.height(),
                        });
                    }
                    WindowEvent::CloseRequested => {
                        let mut window_close_requested_events = world
                            .get_resource_mut::<Events<WindowCloseRequested>>()
                            .unwrap();
                        window_close_requested_events.send(WindowCloseRequested { id: window_id });
                    }
                    WindowEvent::KeyboardInput { ref input, .. } => {
                        let mut keyboard_input_events =
                            world.get_resource_mut::<Events<KeyboardInput>>().unwrap();
                        keyboard_input_events.send(converters::convert_keyboard_input(input));
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let mut cursor_moved_events =
                            world.get_resource_mut::<Events<CursorMoved>>().unwrap();
                        let winit_window = winit_windows.get_window(window_id).unwrap();
                        let position = position.to_logical(winit_window.scale_factor());
                        let inner_size = winit_window
                            .inner_size()
                            .to_logical::<f32>(winit_window.scale_factor());

                        // move origin to bottom left
                        let y_position = inner_size.height - position.y;

                        let position = Vec2::new(position.x, y_position);
                        window.update_cursor_position_from_backend(Some(position));

                        cursor_moved_events.send(CursorMoved {
                            id: window_id,
                            position,
                        });
                    }
                    WindowEvent::CursorEntered { .. } => {
                        let mut cursor_entered_events =
                            world.get_resource_mut::<Events<CursorEntered>>().unwrap();
                        cursor_entered_events.send(CursorEntered { id: window_id });
                    }
                    WindowEvent::CursorLeft { .. } => {
                        let mut cursor_left_events =
                            world.get_resource_mut::<Events<CursorLeft>>().unwrap();
                        window.update_cursor_position_from_backend(None);
                        cursor_left_events.send(CursorLeft { id: window_id });
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        let mut mouse_button_input_events = world
                            .get_resource_mut::<Events<MouseButtonInput>>()
                            .unwrap();
                        mouse_button_input_events.send(MouseButtonInput {
                            button: converters::convert_mouse_button(button),
                            state: converters::convert_element_state(state),
                        });
                    }
                    WindowEvent::MouseWheel { delta, .. } => match delta {
                        event::MouseScrollDelta::LineDelta(x, y) => {
                            let mut mouse_wheel_input_events =
                                world.get_resource_mut::<Events<MouseWheel>>().unwrap();
                            mouse_wheel_input_events.send(MouseWheel {
                                unit: MouseScrollUnit::Line,
                                x,
                                y,
                            });
                        }
                        event::MouseScrollDelta::PixelDelta(p) => {
                            let mut mouse_wheel_input_events =
                                world.get_resource_mut::<Events<MouseWheel>>().unwrap();
                            mouse_wheel_input_events.send(MouseWheel {
                                unit: MouseScrollUnit::Pixel,
                                x: p.x as f32,
                                y: p.y as f32,
                            });
                        }
                    },
                    WindowEvent::Touch(touch) => {
                        let mut touch_input_events =
                            world.get_resource_mut::<Events<TouchInput>>().unwrap();

                        let winit_window = winit_windows.get_window(window_id).unwrap();
                        let mut location = touch.location.to_logical(winit_window.scale_factor());

                        // On a mobile window, the start is from the top while on PC/Linux/OSX from
                        // bottom
                        if cfg!(target_os = "android") || cfg!(target_os = "ios") {
                            let window_height = windows.get_primary().unwrap().height();
                            location.y = window_height - location.y;
                        }
                        touch_input_events.send(converters::convert_touch_input(touch, location));
                    }
                    WindowEvent::ReceivedCharacter(c) => {
                        let mut char_input_events = world
                            .get_resource_mut::<Events<ReceivedCharacter>>()
                            .unwrap();

                        char_input_events.send(ReceivedCharacter {
                            id: window_id,
                            char: c,
                        });
                    }
                    WindowEvent::ScaleFactorChanged {
                        scale_factor,
                        new_inner_size,
                    } => {
                        let mut backend_scale_factor_change_events = world
                            .get_resource_mut::<Events<WindowBackendScaleFactorChanged>>()
                            .unwrap();
                        backend_scale_factor_change_events.send(WindowBackendScaleFactorChanged {
                            id: window_id,
                            scale_factor,
                        });
                        let prior_factor = window.scale_factor();
                        window.update_scale_factor_from_backend(scale_factor);
                        let new_factor = window.scale_factor();
                        if let Some(forced_factor) = window.scale_factor_override() {
                            // If there is a scale factor override, then force that to be used
                            // Otherwise, use the OS suggested size
                            // We have already told the OS about our resize constraints, so
                            // the new_inner_size should take those into account
                            *new_inner_size = winit::dpi::LogicalSize::new(
                                window.requested_width(),
                                window.requested_height(),
                            )
                            .to_physical::<u32>(forced_factor);
                        } else if approx::relative_ne!(new_factor, prior_factor) {
                            let mut scale_factor_change_events = world
                                .get_resource_mut::<Events<WindowScaleFactorChanged>>()
                                .unwrap();

                            scale_factor_change_events.send(WindowScaleFactorChanged {
                                id: window_id,
                                scale_factor,
                            });
                        }

                        let new_logical_width = f64::from(new_inner_size.width) / new_factor;
                        let new_logical_height = f64::from(new_inner_size.height) / new_factor;
                        if approx::relative_ne!(f64::from(window.width()), new_logical_width)
                            || approx::relative_ne!(f64::from(window.height()), new_logical_height)
                        {
                            let mut resize_events =
                                world.get_resource_mut::<Events<WindowResized>>().unwrap();
                            resize_events.send(WindowResized {
                                id: window_id,
                                width: new_logical_width as f32,
                                height: new_logical_height as f32,
                            });
                        }
                        window.update_actual_size_from_backend(
                            new_inner_size.width,
                            new_inner_size.height,
                        );
                    }
                    WindowEvent::Focused(focused) => {
                        window.update_focused_status_from_backend(focused);
                        let mut focused_events =
                            world.get_resource_mut::<Events<WindowFocused>>().unwrap();
                        focused_events.send(WindowFocused {
                            id: window_id,
                            focused,
                        });
                    }
                    WindowEvent::DroppedFile(path_buf) => {
                        let mut events =
                            world.get_resource_mut::<Events<FileDragAndDrop>>().unwrap();
                        events.send(FileDragAndDrop::DroppedFile {
                            id: window_id,
                            path_buf,
                        });
                    }
                    WindowEvent::HoveredFile(path_buf) => {
                        let mut events =
                            world.get_resource_mut::<Events<FileDragAndDrop>>().unwrap();
                        events.send(FileDragAndDrop::HoveredFile {
                            id: window_id,
                            path_buf,
                        });
                    }
                    WindowEvent::HoveredFileCancelled => {
                        let mut events =
                            world.get_resource_mut::<Events<FileDragAndDrop>>().unwrap();
                        events.send(FileDragAndDrop::HoveredFileCancelled { id: window_id });
                    }
                    WindowEvent::Moved(position) => {
                        let position = ivec2(position.x, position.y);
                        window.update_actual_position_from_backend(position);
                        let mut events = world.get_resource_mut::<Events<WindowMoved>>().unwrap();
                        events.send(WindowMoved {
                            id: window_id,
                            position,
                        });
                    }
                    _ => {}
                }
            }
            event::Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                let mut mouse_motion_events =
                    app.world.get_resource_mut::<Events<MouseMotion>>().unwrap();
                mouse_motion_events.send(MouseMotion {
                    delta: Vec2::new(delta.0 as f32, delta.1 as f32),
                });
            }
            event::Event::Suspended => {
                active = false;
            }
            event::Event::Resumed => {
                active = true;
            }
            event::Event::MainEventsCleared => {
                handle_create_window_events(
                    &mut app.world,
                    event_loop,
                    &mut create_window_event_reader,
                );
                if active {
                    app.update();
                }
            }
            _ => (),
        }
    };
    if should_return_from_run {
        run_return(&mut event_loop, event_handler);
    } else {
        run(event_loop, event_handler);
    }
}

fn handle_create_window_events(
    world: &mut World,
    event_loop: &EventLoopWindowTarget<()>,
    create_window_event_reader: &mut ManualEventReader<CreateWindow>,
) {
    let world = world.cell();
    let mut winit_windows = world.get_resource_mut::<WinitWindows>().unwrap();
    let mut windows = world.get_resource_mut::<Windows>().unwrap();
    let create_window_events = world.get_resource::<Events<CreateWindow>>().unwrap();
    let mut window_created_events = world.get_resource_mut::<Events<WindowCreated>>().unwrap();
    for create_window_event in create_window_event_reader.iter(&create_window_events) {
        let window = winit_windows.create_window(
            event_loop,
            create_window_event.id,
            &create_window_event.descriptor,
        );
        windows.add(window);
        window_created_events.send(WindowCreated {
            id: create_window_event.id,
        });
    }
}
