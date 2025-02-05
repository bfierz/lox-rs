use crate::tokens::{Token, TokenType};

pub struct Scanner {
    source: String,
    pub had_error: bool,

    tokens: Vec<Token>,

    start: i32,
    current: i32,
    line: i32,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner { source, had_error: false, tokens: Vec::new(), start: 0, current: 0, line: 1 }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            // We are at the beginning of the next lexeme.
            self.start = self.current;
            self.scan_token();
        }

        // Placeholder: add a single dummy token
        self.tokens.push(Token::new(TokenType::Eof, "".to_string(), -1, self.line));
        &self.tokens
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            _ => self.error(self.line, "Unexpected character."),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len() as i32
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current as usize).unwrap();
        self.current += 1;
        c
    }

    fn add_token(&mut self, token_type:TokenType) {
        self.add_token_with_literal(token_type, -1);
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: i32) {
        let text = &self.source[self.start as usize..self.current as usize];
        self.tokens.push(Token::new(token_type, text.to_string(), literal, self.line));
    }

    fn error(&mut self, line: i32, message: &str) {
        Self::report(self, line, "", message);
    }

    fn report(&mut self, line: i32, location: &str, message: &str) {
        println!("[line  {}] Error {}: {}", line, location, message);
        self.had_error = true;
    }
}
