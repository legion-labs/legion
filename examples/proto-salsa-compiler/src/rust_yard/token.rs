#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Identifier(String), // Can be a function or an argument
    Comma,
    LeftParenthesis,
    RightParenthesis,
    Whitespace,
}
