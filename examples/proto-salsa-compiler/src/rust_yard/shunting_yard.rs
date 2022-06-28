/*
 * Rustyard - Simon Whitehead, 2016
 */

use std;

use super::{lexer, token};

/// The ShuntingYard struct transforms an expression
/// to a 64-bit floating point value
pub struct ShuntingYard<'a> {
    lexer: lexer::Lexer<'a>,
    output_queue: Vec<token::Token>,
    stack: Vec<token::Token>,
    errors: Vec<String>,
}

impl<'a> ShuntingYard<'a> {
    pub fn new() -> Self {
        Self {
            lexer: lexer::Lexer::new(),
            output_queue: vec![],
            stack: vec![],
            errors: vec![],
        }
    }

    /// Parse towards a Reverse Polish Notation represented
    /// by the `output_queue`.
    /// Return the parsing tokens
    pub fn parse(&mut self, raw_input: &'a str) -> Result<Vec<token::Token>, Vec<String>> {
        // Clear out everything
        self.output_queue.clear();
        self.stack.clear();
        self.errors.clear();

        // Lex the input
        self.lexer.lex(raw_input);

        // If there were Lexer errors, add them now.
        let lexer_errors = self.lexer.errors.clone();
        self.errors.extend(lexer_errors);

        // Transform the Lexer input via the Shunting Yard algorithm
        self.transform();

        // If there are lexer errors, return early with them
        if !self.errors.is_empty() {
            println!("Errors: {:?}", self.errors);
            return Err(self.errors.clone());
        }

        Ok(self.output_queue.clone())
    }

    // Transforms the input from the Lexer in to the output_queue
    // and stack based on the Shunting Yard algorithm
    fn transform(&mut self) {
        // Iterate over each token and move it based on the algorithm
        for tok in &self.lexer.ast {
            // If the token is a number, then add it to the output queue
            match *tok {
                token::Token::Identifier(_) => self.stack.push(tok.clone()),
                token::Token::LeftParenthesis => self.stack.push(token::Token::LeftParenthesis),
                token::Token::RightParenthesis => loop {
                    match self.stack.last() {
                        Some(&token::Token::LeftParenthesis) => {
                            self.stack.pop().unwrap();
                            break;
                        }
                        None => {
                            self.errors.push("Unbalanced parenthesis".to_string());
                            break;
                        }
                        _ => self.output_queue.push(self.stack.pop().unwrap()),
                    }
                },
                token::Token::Comma => loop {
                    match self.stack.last() {
                        Some(&token::Token::LeftParenthesis) => {
                            break;
                        }
                        _ => {
                            if let Some(tok) = self.stack.pop() {
                                self.output_queue.push(tok);
                            } else {
                                self.errors.push("Syntax error.".to_string());
                                break;
                            }
                        }
                    }
                },
                token::Token::Whitespace => (),
            }
        }

        // Are there any operators left on the stack?
        while !self.stack.is_empty() {
            // Pop them off and push them to the output_queue
            let op = self.stack.pop();
            match op {
                Some(token::Token::LeftParenthesis) => {
                    println!("Left mismatch");
                    self.errors.push("Unbalanced parenthesis".to_string());
                    break;
                }
                Some(token::Token::RightParenthesis) => {
                    println!("Right mismatch");
                    self.errors.push("Unbalanced parenthesis".to_string());
                    break;
                }
                _ => self.output_queue.push(op.unwrap()),
            }
        }
    }

    /// to_string_ast returns the string representation of the
    /// Lexer tokens.
    pub fn to_string_ast(&self) -> String {
        let mut result = String::new(); // String to output the result to

        // Loop over each item in the AST and print a String representation of it
        for tok in &self.lexer.ast {
            match *tok {
                token::Token::Identifier(ref f) => {
                    result.push_str(&f.downcast_ref::<String>().unwrap().clone()[..]);
                }
                token::Token::LeftParenthesis => result.push('('),
                token::Token::RightParenthesis => result.push(')'),
                token::Token::Comma => result.push(','),
                token::Token::Whitespace => (),
            };

            if *tok != token::Token::Whitespace {
                result.push(' '); // Space separated
            }
        }

        // Return the result
        result
    }
}

impl<'a> std::string::ToString for ShuntingYard<'a> {
    /// `to_string` returns the string representation of the Shunting Yard
    /// algorithm in Reverse Polish Notation.
    fn to_string(&self) -> String {
        let mut result = String::new(); // String to output the result

        // Iterate over the output queue and print each one to the result
        for tok in &self.output_queue {
            match *tok {
                token::Token::Identifier(ref f) => {
                    result.push_str(&f.downcast_ref::<String>().unwrap().clone()[..]);
                }
                token::Token::LeftParenthesis => result.push('('),
                token::Token::RightParenthesis => result.push(')'),
                token::Token::Comma => result.push(','),
                token::Token::Whitespace => (),
            };

            if *tok != token::Token::Whitespace {
                result.push(' '); // Space separated
            }
        }

        result.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::ShuntingYard;

    #[test]
    fn test_parse_simple() {
        let mut shunting_yard = ShuntingYard::new();
        shunting_yard.parse("atlas()").unwrap();
        assert_eq!(shunting_yard.to_string(), "atlas");
    }

    #[test]
    fn test_parse_nested() {
        let mut shunting_yard = ShuntingYard::new();
        shunting_yard.parse("atlas(read())").unwrap();
        println!("{}", shunting_yard.to_string());
        assert_eq!(shunting_yard.to_string(), "read atlas");
    }

    #[test]
    fn test_parse_argument() {
        let mut shunting_yard = ShuntingYard::new();
        shunting_yard.parse("atlas(atlas.atlas)").unwrap();
        assert_eq!(shunting_yard.to_string(), "atlas.atlas atlas");
    }

    #[test]
    fn test_parse_arguments() {
        let mut shunting_yard = ShuntingYard::new();
        shunting_yard.parse("atlas(atlas.atlas, 0, 9)").unwrap();
        assert_eq!(shunting_yard.to_string(), "atlas.atlas 0 9 atlas");
    }

    #[test]
    fn test_parse_nested_arguments() {
        let mut shunting_yard = ShuntingYard::new();
        shunting_yard
            .parse("atlas(read(atlas.atlas), 0, 9)")
            .unwrap();
        assert_eq!(shunting_yard.to_string(), "atlas.atlas read 0 9 atlas");
    }
}
