use crate::tokens::{Token, TokenType};

pub struct Scanner {
    source: String,
    pub had_error: bool,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner { source, had_error: false }
    }

    pub fn scan_tokens(&self) -> Vec<Token> {
        let mut tokens = Vec::new();

        // Placeholder: add a single dummy token
        tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: "".to_string(),
            literal: -1,
            line: -1
        });

        tokens
    }

    fn error(&mut self, line: i32, message: &str) {
        Self::report(self, line, "", message);
    }

    fn report(&mut self, line: i32, location: &str, message: &str) {
        println!("[line  {}] Error {}: {}", line, location, message);
        self.had_error = true;
    }
}
