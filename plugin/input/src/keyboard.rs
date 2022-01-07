use lgn_app::EventReader;
use lgn_ecs::system::ResMut;

use crate::{ElementState, Input};

/// A key input event from a keyboard device
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyboardInput {
    pub scan_code: u32,
    pub key_code: Option<KeyCode>,
    pub state: ElementState,
}

/// Updates the Input<KeyCode> resource with the latest `KeyboardInput` events
pub fn keyboard_input_system(
    mut keyboard_input: ResMut<'_, Input<KeyCode>>,
    mut keyboard_input_events: EventReader<'_, '_, KeyboardInput>,
) {
    keyboard_input.clear();
    for event in keyboard_input_events.iter() {
        if let KeyboardInput {
            key_code: Some(key_code),
            state,
            ..
        } = event
        {
            match state {
                ElementState::Pressed => keyboard_input.press(*key_code),
                ElementState::Released => keyboard_input.release(*key_code),
            }
        }
    }
}

/// The key code of a keyboard input.
#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum KeyCode {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    /// The Escape key, next to F1.
    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    Back,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,

    AbntC1,
    AbntC2,
    NumpadAdd,
    Apostrophe,
    Apps,
    Asterisk,
    Plus,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    NumpadDecimal,
    NumpadDivide,
    Equals,
    Grave,
    Kana,
    Kanji,
    /// The left alt key. Maps to left option on Mac.
    LAlt,
    LBracket,
    LControl,
    LShift,
    /// The left Windows key. Maps to left Command on Mac.
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    NumpadMultiply,
    Mute,
    MyComputer,
    NavigateForward,  // also called "Prior"
    NavigateBackward, // also called "Next"
    NextTrack,
    NoConvert,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    Oem102,
    Period,
    PlayPause,
    Power,
    PrevTrack,
    /// The right alt key. Maps to right option on Mac.
    RAlt,
    RBracket,
    RControl,
    RShift,
    /// The right Windows key. Maps to right Command on Mac.
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    NumpadSubtract,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}

impl From<KeyCode> for char {
    fn from(key: KeyCode) -> Self {
        match key {
            KeyCode::Key1 => '1',
            KeyCode::Key2 => '2',
            KeyCode::Key3 => '3',
            KeyCode::Key4 => '4',
            KeyCode::Key5 => '5',
            KeyCode::Key6 => '6',
            KeyCode::Key7 => '7',
            KeyCode::Key8 => '8',
            KeyCode::Key9 => '9',
            KeyCode::Key0 => '0',

            KeyCode::A => 'a',
            KeyCode::B => 'b',
            KeyCode::C => 'c',
            KeyCode::D => 'd',
            KeyCode::E => 'e',
            KeyCode::F => 'f',
            KeyCode::G => 'g',
            KeyCode::H => 'h',
            KeyCode::I => 'i',
            KeyCode::J => 'j',
            KeyCode::K => 'k',
            KeyCode::L => 'l',
            KeyCode::M => 'm',
            KeyCode::N => 'n',
            KeyCode::O => 'o',
            KeyCode::P => 'p',
            KeyCode::Q => 'q',
            KeyCode::R => 'r',
            KeyCode::S => 's',
            KeyCode::T => 't',
            KeyCode::U => 'u',
            KeyCode::V => 'v',
            KeyCode::W => 'w',
            KeyCode::X => 'x',
            KeyCode::Y => 'y',
            KeyCode::Z => 'z',

            //Escape,

            //F1,
            //F2,
            //F3,
            //F4,
            //F5,
            //F6,
            //F7,
            //F8,
            //F9,
            //F10,
            //F11,
            //F12,
            //F13,
            //F14,
            //F15,
            //F16,
            //F17,
            //F18,
            //F19,
            //F20,
            //F21,
            //F22,
            //F23,
            //F24,

            //Snapshot,
            //Scroll,
            //Pause,

            //Insert,
            //Home,
            //Delete,
            //End,
            //PageDown,
            //PageUp,

            //Left,
            //Up,
            //Right,
            //Down,

            //Back,
            //Return,
            KeyCode::Space => ' ',

            //Compose,

            //Caret,

            //Numlock,
            //Numpad0,
            //Numpad1,
            //Numpad2,
            //Numpad3,
            //Numpad4,
            //Numpad5,
            //Numpad6,
            //Numpad7,
            //Numpad8,
            //Numpad9,

            //AbntC1,
            //AbntC2,
            //NumpadAdd,
            KeyCode::Apostrophe => '\'',
            //Apps,
            KeyCode::Asterisk => '*',
            KeyCode::Plus => '+',
            KeyCode::At => '@',
            //Ax,
            KeyCode::Backslash => '\\',
            //Calculator,
            //KeyCode::Capital => '',
            KeyCode::Colon => ':',
            KeyCode::Comma => ',',
            //KeyCode::Convert => '',
            //KeyCode::NumpadDecimal => '',
            //KeyCode::NumpadDivide => '',
            KeyCode::Equals => '=',
            KeyCode::Grave => '`',
            //KeyCode::Kana => '',
            //KeyCode::Kanji => '',
            //KeyCode::LAlt => '',
            //KeyCode::LBracket => '(',
            //KeyCode::LControl => '',
            //KeyCode::LShift => '',
            //KeyCode::LWin => '',
            //KeyCode::Mail => '',
            //KeyCode::MediaSelect => '',
            //KeyCode::MediaStop => '',
            KeyCode::Minus => '-',
            //KeyCode::NumpadMultiply => '',
            //KeyCode::Mute => '',
            //KeyCode::MyComputer => '',
            //KeyCode::NavigateForward => '',  // also called "Prior"
            //KeyCode::NavigateBackward => '', // also called "Next"
            //KeyCode::NextTrack => '',
            //KeyCode::NoConvert => '',
            //KeyCode::NumpadComma => '',
            //KeyCode::NumpadEnter => '',
            //KeyCode::NumpadEquals => '',
            //KeyCode::Oem102 => '',
            KeyCode::Period => '.',
            //KeyCode::PlayPause => '',
            //KeyCode::Power => '',
            //KeyCode::PrevTrack => '',
            //KeyCode::RAlt => '',
            KeyCode::RBracket => ')',
            //KeyCode::RControl => '',
            //KeyCode::RShift => '',
            //KeyCode::RWin => '',
            KeyCode::Semicolon => ';',
            KeyCode::Slash => '/',
            //KeyCode::Sleep => '',
            //KeyCode::Stop => '',
            //KeyCode::NumpadSubtract => '',
            //KeyCode::Sysrq => '',
            //KeyCode::Tab => '',
            KeyCode::Underline => '_',
            //KeyCode::Unlabeled => '',
            //KeyCode::VolumeDown => '',
            //KeyCode::VolumeUp => '',
            //KeyCode::Wake => '',
            //KeyCode::WebBack => '',
            //KeyCode::WebFavorites => '',
            //KeyCode::WebForward => '',
            //KeyCode::WebHome => '',
            //KeyCode::WebRefresh => '',
            //KeyCode::WebSearch => '',
            //KeyCode::WebStop => '',
            //KeyCode::Yen => '',
            //KeyCode::Copy => '',
            //KeyCode::Paste => '',
            //KeyCode::Cut => '',
            _ => 0 as Self,
        }
    }
}
