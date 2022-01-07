use std::{fmt::Display, io::Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub(crate) const ACTION_STEP_COLOR: Color = Color::Green;
pub(crate) const IGNORE_STEP_COLOR: Color = Color::Yellow;

pub fn print_step(color: Color, action: &str, description: impl Display) {
    if atty::is(atty::Stream::Stdout) {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        stdout
            .set_color(
                ColorSpec::new()
                    .set_fg(Some(color))
                    .set_intense(true)
                    .set_bold(true),
            )
            .unwrap();
        write!(
            &mut stdout,
            "{}{}",
            (0..(12 - action.len())).map(|_| " ").collect::<String>(),
            action
        )
        .unwrap();
        stdout.reset().unwrap();
        writeln!(&mut stdout, " {}", description).unwrap();
    } else {
        println!(
            "{}{} {}",
            (0..(12 - action.len())).map(|_| " ").collect::<String>(),
            action,
            description
        );
    }
}

/// Prints an action step, with a green action verb followed by the subject.
#[macro_export]
macro_rules! action_step {
    ($action:expr, $description:expr $(,)?) => {
        $crate::term::print_step($crate::term::ACTION_STEP_COLOR, $action, $description)
    };
    ($action:expr, $fmt:expr, $($arg:tt)*) => {
        action_step!($action, format!($fmt, $($arg)*))
    };
}

/// Prints an ignore step, with a yellow action verb followed by the subject.
#[macro_export]
macro_rules! ignore_step {
    ($action:expr, $description:expr $(,)?) => {
        $crate::term::print_step($crate::term::IGNORE_STEP_COLOR, $action, $description)
    };
    ($action:expr, $fmt:expr, $($arg:tt)*) => {
        ignore_step!($action, format!($fmt, $($arg)*))
    };
}
