//! egui wrapper plugin

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

use egui::{Event, Key, RawInput};
use legion_app::prelude::*;
use legion_ecs::prelude::*;
use legion_input::{
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButton, MouseButtonInput, MouseWheel},
};
use legion_window::{CursorMoved, WindowCreated, WindowResized, WindowScaleFactorChanged, Windows};

#[derive(Default)]
pub struct Egui {
    pub ctx: egui::CtxRef,
}

#[derive(SystemLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EguiLabels {
    GatherInput,
    BeginFrame,
}

#[derive(Default)]
pub struct EguiPlugin {
    has_window: bool,
}

impl EguiPlugin {
    pub fn new(has_window: bool) -> Self {
        Self { has_window }
    }
}

impl Plugin for EguiPlugin {
    fn build(&self, app: &mut App) {
        if self.has_window {
            app.add_startup_system(on_window_created.system());
        }

        app.insert_resource(Egui::default());
        app.insert_resource(RawInput::default());
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            gather_input.system().label(EguiLabels::GatherInput),
        );
        if self.has_window {
            app.add_system_to_stage(
                CoreStage::PreUpdate,
                gather_input_window
                    .system()
                    .after(EguiLabels::GatherInput)
                    .before(EguiLabels::BeginFrame),
            );
        }
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            begin_frame
                .system()
                .label(EguiLabels::BeginFrame)
                .after(EguiLabels::GatherInput),
        );
    }
}

#[allow(clippy::needless_pass_by_value)]
fn on_window_created(
    mut egui: ResMut<'_, Egui>,
    mut ev_wnd_created: EventReader<'_, '_, WindowCreated>,
    wnd_list: Res<'_, Windows>,
) {
    let mut size = egui::vec2(1280.0, 720.0);
    let mut pixels_per_point = 1.0;
    for ev in ev_wnd_created.iter() {
        let wnd = wnd_list.get(ev.id).unwrap();
        #[allow(clippy::cast_precision_loss)]
        {
            size = egui::vec2(wnd.physical_width() as f32, wnd.physical_height() as f32);
        }
        pixels_per_point = wnd.scale_factor();
    }
    // We need to run begin_frame at least once so we have the font texture content
    egui.ctx.begin_frame(RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), size)),
        pixels_per_point: Some(pixels_per_point as f32),
        ..RawInput::default()
    });
    #[allow(unused_must_use)]
    {
        egui.ctx.end_frame();
    }
}

fn pointer_button_from_mouse_button(mouse_button: MouseButton) -> egui::PointerButton {
    match mouse_button {
        MouseButton::Left => egui::PointerButton::Primary,
        MouseButton::Middle => egui::PointerButton::Middle,
        _ => egui::PointerButton::Secondary,
    }
}

fn key_from_key_code(key: KeyCode) -> Option<Key> {
    match key {
        KeyCode::Down => Some(Key::ArrowDown),
        KeyCode::Left => Some(Key::ArrowLeft),
        KeyCode::Right => Some(Key::ArrowRight),
        KeyCode::Up => Some(Key::ArrowUp),

        KeyCode::Escape => Some(Key::Escape),
        KeyCode::Tab => Some(Key::Tab),
        KeyCode::Back => Some(Key::Backspace),
        KeyCode::Return => Some(Key::Enter),
        KeyCode::Space => Some(Key::Space),

        KeyCode::Insert => Some(Key::Insert),
        KeyCode::Delete => Some(Key::Delete),
        KeyCode::Home => Some(Key::Home),
        KeyCode::End => Some(Key::End),
        KeyCode::PageUp => Some(Key::PageUp),
        KeyCode::PageDown => Some(Key::PageDown),

        KeyCode::Numpad0 | KeyCode::Key0 => Some(Key::Num0),
        KeyCode::Numpad1 | KeyCode::Key1 => Some(Key::Num1),
        KeyCode::Numpad2 | KeyCode::Key2 => Some(Key::Num2),
        KeyCode::Numpad3 | KeyCode::Key3 => Some(Key::Num3),
        KeyCode::Numpad4 | KeyCode::Key4 => Some(Key::Num4),
        KeyCode::Numpad5 | KeyCode::Key5 => Some(Key::Num5),
        KeyCode::Numpad6 | KeyCode::Key6 => Some(Key::Num6),
        KeyCode::Numpad7 | KeyCode::Key7 => Some(Key::Num7),
        KeyCode::Numpad8 | KeyCode::Key8 => Some(Key::Num8),
        KeyCode::Numpad9 | KeyCode::Key9 => Some(Key::Num9),

        KeyCode::A => Some(Key::A), // Used for cmd+A (select All)
        KeyCode::B => Some(Key::B),
        KeyCode::C => Some(Key::C),
        KeyCode::D => Some(Key::D),
        KeyCode::E => Some(Key::E),
        KeyCode::F => Some(Key::F),
        KeyCode::G => Some(Key::G),
        KeyCode::H => Some(Key::H),
        KeyCode::I => Some(Key::I),
        KeyCode::J => Some(Key::J),
        KeyCode::K => Some(Key::K), // Used for ctrl+K (delete text after cursor)
        KeyCode::L => Some(Key::L),
        KeyCode::M => Some(Key::M),
        KeyCode::N => Some(Key::N),
        KeyCode::O => Some(Key::O),
        KeyCode::P => Some(Key::P),
        KeyCode::Q => Some(Key::Q),
        KeyCode::R => Some(Key::R),
        KeyCode::S => Some(Key::S),
        KeyCode::T => Some(Key::T),
        KeyCode::U => Some(Key::U), // Used for ctrl+U (delete text before cursor)
        KeyCode::V => Some(Key::V),
        KeyCode::W => Some(Key::W), // Used for ctrl+W (delete previous word)
        KeyCode::X => Some(Key::X),
        KeyCode::Y => Some(Key::Y),
        KeyCode::Z => Some(Key::Z), // Used for cmd+Z (undo)
        _ => None,
    }
}

fn gather_input(
    raw_input: ResMut<'_, RawInput>,
    mut cursor_button: EventReader<'_, '_, MouseButtonInput>,
    mut mouse_wheel_events: EventReader<'_, '_, MouseWheel>,
    mut keyboard_input_events: EventReader<'_, '_, KeyboardInput>,
) {
    let mut scroll_delta = egui::vec2(0.0, 0.0);
    for mouse_wheel_event in mouse_wheel_events.iter() {
        scroll_delta.x += mouse_wheel_event.x;
        scroll_delta.y += mouse_wheel_event.y;
    }

    // TODO: zoom_delta
    // TODO: screen_rect
    // TODO: pixels_per_point
    // TODO: time
    // TODO: predicted_dt: f32,
    // TODO: modifiers: Modifiers,
    // TODO: hovered_files: Vec<HoveredFile>,
    // TODO: dropped_files: Vec<DroppedFile>,

    // Events
    let mut events: Vec<Event> = Vec::new();

    // TODO: Copy,
    // TODO: Cut,
    // TODO: PointerGone,
    // TODO: CompositionStart,
    // TODO: CompositionUpdate(String),
    // TODO: CompositionEnd(String),
    // TODO: Touch

    for cursor_button_event in cursor_button.iter() {
        events.push(Event::PointerButton {
            pos: egui::pos2(cursor_button_event.pos.x, cursor_button_event.pos.y),
            button: pointer_button_from_mouse_button(cursor_button_event.button),
            pressed: cursor_button_event.state.is_pressed(),
            modifiers: egui::Modifiers::default(), // TODO
        });
    }

    for keyboard_input_event in keyboard_input_events.iter() {
        if let Some(key) = key_from_key_code(keyboard_input_event.key_code.unwrap()) {
            events.push(Event::Key {
                key,
                pressed: keyboard_input_event.state.is_pressed(),
                modifiers: egui::Modifiers::default(), // TODO
            });
        }
        if keyboard_input_event.state.is_pressed() {
            let ch: char = keyboard_input_event.key_code.unwrap().into();
            if ch != 0 as char {
                events.push(Event::Text(String::from(ch)));
            }
        }
    }

    let raw_input = raw_input.into_inner();
    raw_input.clone_from(&RawInput {
        scroll_delta,
        events,
        ..RawInput::default()
    });
}

fn gather_input_window(
    mut raw_input: ResMut<'_, RawInput>,
    mut cursor_moved: EventReader<'_, '_, CursorMoved>,
    mut scale_factor_changed: EventReader<'_, '_, WindowScaleFactorChanged>,
    mut window_resized_events: EventReader<'_, '_, WindowResized>,
) {
    for cursor_moved_event in cursor_moved.iter() {
        raw_input.events.push(Event::PointerMoved(egui::pos2(
            cursor_moved_event.position.x,
            cursor_moved_event.position.y,
        )));
    }
    for scale_factor_event in scale_factor_changed.iter() {
        raw_input.pixels_per_point = Some(scale_factor_event.scale_factor as f32);
    }
    for window_resized_event in window_resized_events.iter() {
        raw_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(window_resized_event.width, window_resized_event.height),
        ));
    }
}

#[allow(clippy::needless_pass_by_value)]
fn begin_frame(mut egui: ResMut<'_, Egui>, raw_input: Res<'_, RawInput>) {
    egui.ctx.begin_frame(raw_input.to_owned());

    egui::Window::new("Debug").show(&egui.ctx, |ui| {
        egui.ctx.settings_ui(ui);
        ui.label(egui.ctx.texture().version);
    });
}
