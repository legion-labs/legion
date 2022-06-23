use std::iter;
use std::str;

/// PeekableStringIterator owns an iterator that advances
/// through some string input. The idea here is that the Lexer
/// should not actually own the iterator that iterates through
/// the input - so it is wrapper in this struct.
pub struct PeekableStringIterator<'a> {
    iter: iter::Peekable<str::Chars<'a>>
}

impl<'a> PeekableStringIterator<'a> {
    pub fn new() -> PeekableStringIterator<'a> {
        PeekableStringIterator {
            iter: "".chars().peekable()
        }
    }

    pub fn set_input(&mut self, raw_input: &'a str) {
        self.iter = raw_input.chars().peekable();
    }

    pub fn advance(&mut self) {
        self.iter.next();
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.iter.peek()
    }
}
