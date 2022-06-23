//! Rustyard - Simon Whitehead, 2016.
//!
//! My first proper attempt at Rust code, Rustyard is
//! an implementation of the Shunting Yard algorithm and
//! can calculate the value of mathematical expressions
//! passed to it as strings.

pub mod shunting_yard;
pub mod token;

mod lexer;
mod peekable_string_iterator;
mod rpn_calculator;

// Exported types
pub use self::shunting_yard::ShuntingYard;
