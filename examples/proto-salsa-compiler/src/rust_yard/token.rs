use std::sync::Arc;

use crate::compiler::AnyEq;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Identifier(Arc<Box<dyn AnyEq>>), // Can be a function or an argument
    Comma,
    LeftParenthesis,
    RightParenthesis,
    Whitespace,
}
