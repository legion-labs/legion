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

use egui::{CtxRef, Event, RawInput};
use legion_app::prelude::*;
use legion_ecs::prelude::*;
use legion_input::{
    mouse::{MouseButton, MouseButtonInput},
    ElementState,
};
use legion_window::{CursorMoved, WindowCreated, Windows};

pub struct Egui {
    pub ctx: egui::CtxRef,
}

#[derive(Default)]
pub struct EguiPlugin {}

impl Plugin for EguiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(on_window_created.system())
            .add_system_to_stage(CoreStage::PreUpdate, begin_frame.system());
    }
}

fn on_window_created(
    mut commands: Commands,
    mut ev_wnd_created: EventReader<WindowCreated>,
    wnd_list: Res<Windows>,
) {
    let mut size = egui::vec2(1280.0, 720.0);
    let mut pixels_per_point = 1.0;
    for ev in ev_wnd_created.iter() {
        let wnd = wnd_list.get(ev.id).unwrap();
        size = egui::vec2(wnd.physical_width() as f32, wnd.physical_height() as f32);
        pixels_per_point = 1.0 / wnd.scale_factor();
    }
    let mut ctx = egui::CtxRef::default();
    // We need to run begin_frame at least once so we have the font texture content
    ctx.begin_frame(RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), size)),
        pixels_per_point: Some(pixels_per_point as f32),
        ..RawInput::default()
    });
    ctx.end_frame();
    commands.insert_resource(Egui { ctx });
}

fn begin_frame(
    mut egui: ResMut<'_, Egui>,
    mut cursor_moved: EventReader<'_, '_, CursorMoved>,
    mut cursor_button: EventReader<'_, '_, MouseButtonInput>,
) {
    /*pub struct RawInput {
        /// How many points (logical pixels) the user scrolled
        pub scroll_delta: Vec2,

        /// Zoom scale factor this frame (e.g. from ctrl-scroll or pinch gesture).
        /// * `zoom = 1`: no change (default).
        /// * `zoom < 1`: pinch together
        /// * `zoom > 1`: pinch spread
        pub zoom_delta: f32,

        /// Position and size of the area that egui should use.
        /// Usually you would set this to
        ///
        /// `Some(Rect::from_pos_size(Default::default(), screen_size))`.
        ///
        /// but you could also constrain egui to some smaller portion of your window if you like.
        ///
        /// `None` will be treated as "same as last frame", with the default being a very big area.
        pub screen_rect: Option<Rect>,

        /// Also known as device pixel ratio, > 1 for high resolution screens.
        /// If text looks blurry you probably forgot to set this.
        /// Set this the first frame, whenever it changes, or just on every frame.
        pub pixels_per_point: Option<f32>,

        /// Monotonically increasing time, in seconds. Relative to whatever. Used for animations.
        /// If `None` is provided, egui will assume a time delta of `predicted_dt` (default 1/60 seconds).
        pub time: Option<f64>,

        /// Should be set to the expected time between frames when painting at vsync speeds.
        /// The default for this is 1/60.
        /// Can safely be left at its default value.
        pub predicted_dt: f32,

        /// Which modifier keys are down at the start of the frame?
        pub modifiers: Modifiers,

        /// In-order events received this frame.
        ///
        /// There is currently no way to know if egui handles a particular event,
        /// but you can check if egui is using the keyboard with [`crate::Context::wants_keyboard_input`]
        /// and/or the pointer (mouse/touch) with [`crate::Context::is_using_pointer`].
        pub events: Vec<Event>,

        /// Dragged files hovering over egui.
        pub hovered_files: Vec<HoveredFile>,

        /// Dragged files dropped into egui.
        ///
        /// Note: when using `eframe` on Windows you need to enable
        /// drag-and-drop support using `epi::NativeOptions`.
        pub dropped_files: Vec<DroppedFile>,
    }*/
    /*pub enum Event {
        /// The integration detected a "copy" event (e.g. Cmd+C).
        Copy,
        /// The integration detected a "cut" event (e.g. Cmd+X).
        Cut,
        /// Text input, e.g. via keyboard or paste action.
        ///
        /// When the user presses enter/return, do not send a `Text` (just [`Key::Enter`]).
        Text(String),
        Key {
            key: Key,
            pressed: bool,
            modifiers: Modifiers,
        },

        PointerMoved(Pos2),
        PointerButton {
            pos: Pos2,
            button: PointerButton,
            pressed: bool,
            /// The state of the modifier keys at the time of the event
            modifiers: Modifiers,
        },
        /// The mouse left the screen, or the last/primary touch input disappeared.
        ///
        /// This means there is no longer a cursor on the screen for hovering etc.
        ///
        /// On touch-up first send `PointerButton{pressed: false, …}` followed by `PointerLeft`.
        PointerGone,

        /// IME composition start.
        CompositionStart,
        /// A new IME candidate is being suggested.
        CompositionUpdate(String),
        /// IME composition ended with this final result.
        CompositionEnd(String),

        /// On touch screens, report this *in addition to*
        /// [`Self::PointerMoved`], [`Self::PointerButton`], [`Self::PointerGone`]
        Touch {
            /// Hashed device identifier (if available; may be zero).
            /// Can be used to separate touches from different devices.
            device_id: TouchDeviceId,
            /// Unique identifier of a finger/pen. Value is stable from touch down
            /// to lift-up
            id: TouchId,
            phase: TouchPhase,
            /// Position of the touch (or where the touch was last detected)
            pos: Pos2,
            /// Describes how hard the touch device was pressed. May always be `0` if the platform does
            /// not support pressure sensitivity.
            /// The value is in the range from 0.0 (no pressure) to 1.0 (maximum pressure).
            force: f32,
        },
    }*/
    let mut events: Vec<Event> = Vec::new();
    for cursor_moved_event in cursor_moved.iter() {
        events.push(Event::PointerMoved(egui::pos2(
            cursor_moved_event.position.x / egui.ctx.pixels_per_point(),
            (720.0 - cursor_moved_event.position.y) / egui.ctx.pixels_per_point(),
        )));
    }

    fn from(mouse_button: &MouseButton) -> egui::PointerButton {
        match mouse_button {
            MouseButton::Left => egui::PointerButton::Primary,
            MouseButton::Right => egui::PointerButton::Secondary,
            MouseButton::Middle => egui::PointerButton::Middle,
            _ => unimplemented!(),
        }
    }

    for cursor_button_event in cursor_button.iter() {
        events.push(Event::PointerButton {
            pos: egui::pos2(
                cursor_button_event.pos.x / egui.ctx.pixels_per_point(),
                (720.0 - cursor_button_event.pos.y) / egui.ctx.pixels_per_point(),
            ),
            button: from(&cursor_button_event.button),
            pressed: cursor_button_event.state.is_pressed(),
            modifiers: egui::Modifiers::default(),
        });
    }
    let input = RawInput {
        events,
        ..RawInput::default()
    };
    egui.ctx.begin_frame(input);
}
