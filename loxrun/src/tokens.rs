use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: LiteralTypes,
    pub line: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralTypes {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: LiteralTypes, line: i32) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }

    pub fn new_identifier(lexeme: String, line: i32) -> Self {
        Self {
            token_type: TokenType::Identifier,
            lexeme: lexeme.clone(),
            literal: LiteralTypes::String(lexeme),
            line,
        }
    }

    pub fn new_string(lexeme: String, line: i32) -> Self {
        Self {
            token_type: TokenType::String,
            lexeme: lexeme.clone(),
            literal: LiteralTypes::String(lexeme[1..lexeme.len() - 1].to_string()),
            line,
        }
    }

    pub fn new_number(lexeme: String, line: i32) -> Self {
        let num = lexeme.parse().unwrap();
        Self {
            token_type: TokenType::Number,
            lexeme,
            literal: LiteralTypes::Number(num),
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {} {:?}", self.token_type, self.lexeme, self.literal)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}
