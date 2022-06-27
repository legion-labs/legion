use std::sync::Arc;

use super::peekable_string_iterator as peek;
use super::token;

/// The Lexer struct represents a lexer that tokenizes
/// the string input.
pub struct Lexer<'a> {
    iter: peek::PeekableStringIterator<'a>,
    pub ast: Vec<token::Token>,
    pub errors: Vec<String>,
}

impl<'a> Lexer<'a> {
    pub fn new() -> Lexer<'a> {
        Lexer {
            ast: Vec::new(),
            errors: vec![],
            iter: peek::PeekableStringIterator::new(),
        }
    }

    pub fn lex(&mut self, raw_input: &'a str) {
        // Clear out everything
        self.ast.clear();
        self.errors.clear();

        self.iter.set_input(raw_input);
        self.consume_input();
    }

    // Recursively consume the input
    fn consume_input(&mut self) {
        // Should we skip advancing if a sub method has done it for us?
        let mut skip_advance = false;

        // Peek the next character
        let peeked: Option<char> = self.iter.peek().copied();
        // Decide what to do

        match peeked {
            Some(c) if c.is_whitespace() => {
                self.ast.push(token::Token::Whitespace);
            }
            Some(c) if c == '(' => self.ast.push(token::Token::LeftParenthesis),
            Some(c) if c == ')' => self.ast.push(token::Token::RightParenthesis),
            Some(c) if c == ',' => self.ast.push(token::Token::Comma),
            Some(c) if char_is_identifier(c) => {
                let ident = self.consume_identifier();
                self.ast
                    .push(token::Token::Identifier(Arc::new(Box::new(ident))));
                skip_advance = true;
            }
            _ => return,
        }
        // Advance the iterator and continue consuming the input
        if !skip_advance {
            self.iter.advance();
        }
        self.consume_input();
    }

    // Consumes an identifier until we don't have any other letters available
    fn consume_identifier(&mut self) -> String {
        let mut result = vec![];
        loop {
            match self.iter.peek() {
                Some(c) if char_is_identifier(*c) => result.push(*c),
                _ => break,
            }
            self.iter.advance();
        }

        result.into_iter().collect()
    }
}

fn char_is_identifier(c: char) -> bool {
    !c.is_whitespace() && c != '(' && c != ')' && c != ','
}
