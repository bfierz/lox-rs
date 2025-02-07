use crate::tokens::{LiteralTypes, Token, TokenType};

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
        self.tokens.push(Token::new(TokenType::Eof, "".to_string(), LiteralTypes::Nil, self.line));
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
            '!' => if self.match_next('=') { self.add_token(TokenType::BangEqual) } else { self.add_token(TokenType::Bang) },
            '=' => if self.match_next('=') { self.add_token(TokenType::EqualEqual) } else { self.add_token(TokenType::Equal) },
            '<' => if self.match_next('=') { self.add_token(TokenType::LessEqual) } else { self.add_token(TokenType::Less) },
            '>' => if self.match_next('=') { self.add_token(TokenType::GreaterEqual) } else { self.add_token(TokenType::Greater) },
            '/' => if self.match_next('/') {
                        // A comment goes until the end of the line.
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        self.add_token(TokenType::Slash);
                    },
            ' ' | '\r' | '\t' => (), // Ignore whitespace.
            '\n' => self.line += 1,
            '"' => self.string(),
            _ => if c.is_ascii_digit() {
                self.number();
            } else if self.is_alpha(c) {
                self.identifier();
            } else {
                self.error(self.line, "Unexpected character.")
            },
        }
    }

    fn get_keyword(&self, word: &str) -> Option<TokenType> {
        match word {
            "and" => Some(TokenType::And),
            "class" => Some(TokenType::Class),
            "else" => Some(TokenType::Else),
            "false" => Some(TokenType::False),
            "for" => Some(TokenType::For),
            "fun" => Some(TokenType::Fun),
            "if" => Some(TokenType::If),
            "nil" => Some(TokenType::Nil),
            "or" => Some(TokenType::Or),
            "print" => Some(TokenType::Print),
            "return" => Some(TokenType::Return),
            "super" => Some(TokenType::Super),
            "this" => Some(TokenType::This),
            "true" => Some(TokenType::True),
            "var" => Some(TokenType::Var),
            "while" => Some(TokenType::While),
            _ => None,
        }
    }

    fn identifier(&mut self) {
        while self.is_alpha_numeric(self.peek()) {
            self.advance();
        }
    
        let text = &self.source[self.start as usize..self.current as usize];
        let token_type = self.get_keyword(text);

        match token_type {
            Some(t) => match &t {
                TokenType::True => self.add_token_with_literal(t, LiteralTypes::Bool(true)),
                TokenType::False => self.add_token_with_literal(t, LiteralTypes::Bool(false)),
                _ => self.add_token_with_literal(t, LiteralTypes::Nil),
            },
            None => self.add_token_with_literal(
                TokenType::Identifier,
                LiteralTypes::String(text.to_string()),
            ),
        }
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume the "."
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let value: f64 = self.source[self.start as usize..self.current as usize].parse().unwrap();
        self.add_token_with_literal(TokenType::Number, LiteralTypes::Number(value));
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' { self.line += 1; }
            self.advance();
        }

        if self.is_at_end() {
            self.error(self.line, "Unterminated string.");
            return;
        }

        // The closing ".
        self.advance();

        // Trim the surrounding quotes.
        let value = &self.source[(self.start + 1) as usize..(self.current - 1) as usize];
        self.add_token_with_literal(TokenType::String, LiteralTypes::String(value.to_string()));
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        let c = self.source.chars().nth(self.current as usize).unwrap();
        if c != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current as usize).unwrap()
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() as i32 {
            '\0'
        } else {
            self.source.chars().nth((self.current + 1) as usize).unwrap()
        }
    }

    fn is_alpha_numeric(&self, c: char) -> bool {
        self.is_alpha(c) || c.is_ascii_digit()
    }

    fn is_alpha(&self, c: char) -> bool {
        (c >= 'a' && c <= 'z') ||
        (c >= 'A' && c <= 'Z') ||
         c == '_'
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
        self.add_token_with_literal(token_type, LiteralTypes::Nil);
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: LiteralTypes) {
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
