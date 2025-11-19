use liblox::tokens::{Token, TokenType};

use crate::chunk::Chunk;
use crate::chunk::OpCode;

pub struct Parser {
    tokens: Vec<Token>,
    pub chunk: Chunk,
    current: usize,
    current_id: usize,
}

#[derive(Debug)]
pub struct ParserError {
    pub message: String,
}

enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

struct ParseRule {
    prefix: Option<fn(&mut Parser)>,
    infix: Option<fn(&mut Parser)>,
    precedence: Precedence,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            chunk: Chunk::new(),
            current: 0,
            current_id: 0,
        }
    }

    pub fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        let value: f64 = self.previous().lexeme.parse().unwrap();
        self.emit_constant(value);
    }

    fn literal(&mut self) {
        match self.previous().token_type {
            TokenType::Nil => self.emit_opcode(OpCode::Nil),
            TokenType::True => self.emit_opcode(OpCode::True),
            TokenType::False => self.emit_opcode(OpCode::False),
            _ => {}
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.")
            .unwrap();
    }

    fn unary(&mut self) {
        let operator_token = self.previous();
        self.parse_precedence(Precedence::Unary);

        match operator_token.token_type {
            TokenType::Bang => self.emit_opcode(OpCode::Not),
            TokenType::Minus => self.emit_opcode(OpCode::Negate),
            _ => {}
        }
    }

    fn binary(&mut self) {
        let operator_token = self.previous();
        let precedence = self.get_rule(&operator_token.token_type).precedence;
        self.parse_precedence(precedence);

        match operator_token.token_type {
            TokenType::BangEqual => self.emit_opcodes_2(OpCode::Equal, OpCode::Not),
            TokenType::EqualEqual => self.emit_opcode(OpCode::Equal),
            TokenType::Greater => self.emit_opcode(OpCode::Greater),
            TokenType::GreaterEqual => self.emit_opcodes_2(OpCode::Less, OpCode::Not),
            TokenType::Less => self.emit_opcode(OpCode::Less),
            TokenType::LessEqual => self.emit_opcodes_2(OpCode::Greater, OpCode::Not),
            TokenType::Plus => self.emit_opcode(OpCode::Add),
            TokenType::Minus => self.emit_opcode(OpCode::Subtract),
            TokenType::Star => self.emit_opcode(OpCode::Multiply),
            TokenType::Slash => self.emit_opcode(OpCode::Divide),
            _ => {}
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self
            .get_rule(&self.previous().token_type)
            .prefix
            .expect("Expected prefix rule");
        prefix_rule(self);

        let precedence = precedence as u32;
        while precedence
            <= self
                .get_rule(&self.tokens[self.current].token_type)
                .precedence as u32
        {
            self.advance();
            let infix_rule = self
                .get_rule(&self.previous().token_type)
                .infix
                .expect("Expected infix rule");
            infix_rule(self);
        }
    }

    fn get_rule(&self, token_type: &TokenType) -> ParseRule {
        match token_type {
            TokenType::Number => ParseRule {
                prefix: Some(Parser::number),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Nil => ParseRule {
                prefix: Some(Parser::literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::True => ParseRule {
                prefix: Some(Parser::literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::False => ParseRule {
                prefix: Some(Parser::literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::LeftParen => ParseRule {
                prefix: Some(Parser::grouping),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Minus => ParseRule {
                prefix: Some(Parser::unary),
                infix: Some(Parser::binary),
                precedence: Precedence::Term,
            },
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Term,
            },
            TokenType::Slash => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Factor,
            },
            TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Factor,
            },
            TokenType::Bang => ParseRule {
                prefix: Some(Parser::unary),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::BangEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Equality,
            },
            TokenType::EqualEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Equality,
            },
            TokenType::Greater => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::Less => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Comparison,
            },
            _ => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        }
    }

    pub fn emit_return(&mut self) {
        self.emit_opcode(OpCode::Return);
    }

    fn emit_constant(&mut self, value: f64) {
        let constant_index = self.chunk.add_constant(value);
        if constant_index > u8::MAX as usize {
            panic!("Too many constants in one chunk.");
        }
        self.emit_opcode(OpCode::Constant);
        self.chunk
            .write(constant_index as u8, self.previous().line as u32);
    }

    fn emit_opcode(&mut self, opcode: OpCode) {
        self.chunk
            .write_op_code(opcode, self.previous().line as u32);
    }

    fn emit_opcodes_2(&mut self, opcode: OpCode, opcode2: OpCode) {
        self.chunk
            .write_op_code(opcode, self.previous().line as u32);
        self.chunk
            .write_op_code(opcode2, self.previous().line as u32);
    }

    pub fn match_token(&mut self, tokens: &[TokenType]) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }

        false
    }

    pub fn consume(&mut self, token: TokenType, message: &str) -> Result<Token, ParserError> {
        self.consume_msg(token, message.to_string())
    }

    pub fn consume_msg(&mut self, token: TokenType, message: String) -> Result<Token, ParserError> {
        if self.check(&token) {
            self.advance();
            Ok(self.previous())
        } else if self.is_at_end() {
            let line = self.tokens[self.current].line;
            Err(ParserError {
                message: format!("[line {}] Error at end: {}", line, message),
            })
        } else {
            let line = self.tokens[self.current].line;
            let name = self.tokens[self.current].lexeme.clone();
            Err(ParserError {
                message: format!("[line {}] Error at '{}': {}", line, name, message),
            })
        }
    }

    pub fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    pub fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    pub fn check(&self, token: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.tokens[self.current].token_type == *token
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.tokens[self.current].token_type == TokenType::Eof
    }

    fn next_id(&mut self) -> usize {
        let id = self.current_id;
        self.current_id += 1;
        id
    }
}
