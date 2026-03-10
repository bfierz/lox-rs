use liblox::tokens::{Token, TokenType};

use crate::chunk::Chunk;
use crate::chunk::OpCode;
use crate::chunk::Value;
use crate::hash_table::HashTable;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Parser {
    tokens: Vec<Token>,
    pub chunk: Chunk,
    current: usize,
    current_id: usize,
    panic_occurred: bool,
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
    prefix: Option<fn(&mut Parser, can_assign: bool)>,
    infix: Option<fn(&mut Parser, can_assign: bool)>,
    precedence: Precedence,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, strings: Rc<RefCell<HashTable>>) -> Self {
        Parser {
            tokens,
            chunk: Chunk::new(strings),
            current: 0,
            current_id: 0,
            panic_occurred: false,
        }
    }

    pub fn declaration(&mut self) {
        if self.match_token(&[TokenType::Var]) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_occurred {
            self.synchronize();
        }
    }

    pub fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    pub fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_token(&[TokenType::Equal]) {
            self.expression();
        } else {
            self.emit_opcode(OpCode::Nil);
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )
        .unwrap();

        self.define_variable(global);
    }

    pub fn statement(&mut self) {
        if self.match_token(&[TokenType::Print]) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    pub fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.")
            .unwrap();
        self.emit_opcode(OpCode::Print);
    }

    pub fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")
            .unwrap();
        self.emit_opcode(OpCode::Pop);
    }

    fn synchronize(&mut self) {
        // Reset panic mode so that we can report further errors.
        self.panic_occurred = false;
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.tokens[self.current].token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn string(&mut self, can_assign: bool) {
        let value = self.previous().lexeme.clone();
        self.chunk
            .strings
            .borrow_mut()
            .insert(Value::String(value.clone()), Value::String(value.clone()));
        self.emit_constant(Value::String(value));
    }

    fn number(&mut self, can_assign: bool) {
        let value: f64 = self.previous().lexeme.parse().unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn literal(&mut self, can_assign: bool) {
        match self.previous().token_type {
            TokenType::Nil => self.emit_opcode(OpCode::Nil),
            TokenType::True => self.emit_opcode(OpCode::True),
            TokenType::False => self.emit_opcode(OpCode::False),
            _ => {}
        }
    }

    fn grouping(&mut self, can_assign: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.")
            .unwrap();
    }

    fn unary(&mut self, can_assign: bool) {
        let operator_token = self.previous();
        self.parse_precedence(Precedence::Unary);

        match operator_token.token_type {
            TokenType::Bang => self.emit_opcode(OpCode::Not),
            TokenType::Minus => self.emit_opcode(OpCode::Negate),
            _ => {}
        }
    }

    fn binary(&mut self, can_assign: bool) {
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

    fn variable(&mut self, can_assign: bool) {
        let name = self.previous().lexeme.clone();
        self.named_variable(&name, can_assign);
    }

    fn named_variable(&mut self, name: &str, can_assign: bool) {
        let arg = self.identifier_constant(name);
        if can_assign && self.match_token(&[TokenType::Equal]) {
            self.expression();
            self.emit_opcode(OpCode::SetGlobal);
        } else {
            self.emit_opcode(OpCode::GetGlobal);
        }
        self.chunk.write(arg as u8, self.previous().line as u32);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self
            .get_rule(&self.previous().token_type)
            .prefix
            .expect("Expected prefix rule");

        let precedence = precedence as u32;
        let can_assign = precedence <= Precedence::Assignment as u32;
        prefix_rule(self, can_assign);

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
            infix_rule(self, can_assign);
        }

        if can_assign && self.match_token(&[TokenType::Equal]) {
            self.error_at("Invalid assignment target.").unwrap();
        }
    }

    fn parse_variable(&mut self, error_message: &str) -> usize {
        self.consume(TokenType::Identifier, error_message)
            .expect("Expected variable name.");
        self.identifier_constant(&self.previous().lexeme)
    }

    fn define_variable(&mut self, global: usize) {
        self.emit_opcode(OpCode::DefineGlobal);
        self.chunk.write(global as u8, self.previous().line as u32);
    }

    fn identifier_constant(&mut self, name: &str) -> usize {
        self.make_constant(Value::String(name.to_string()))
    }

    fn get_rule(&self, token_type: &TokenType) -> ParseRule {
        match token_type {
            TokenType::String => ParseRule {
                prefix: Some(Parser::string),
                infix: None,
                precedence: Precedence::None,
            },
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
            TokenType::Identifier => ParseRule {
                prefix: Some(Parser::variable),
                infix: None,
                precedence: Precedence::None,
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

    fn emit_constant(&mut self, value: Value) {
        let constant_index = self.make_constant(value);
        self.emit_opcode(OpCode::Constant);
        self.chunk
            .write(constant_index as u8, self.previous().line as u32);
    }

    fn make_constant(&mut self, value: Value) -> usize {
        let constant_index = self.chunk.add_constant(value);
        if constant_index > u8::MAX as usize {
            panic!("Too many constants in one chunk.");
        }
        constant_index
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
        if self.check(&token) {
            self.advance();
            Ok(self.previous())
        } else {
            self.error_at(message)
        }
    }

    pub fn error_at(&mut self, message: &str) -> Result<Token, ParserError> {
        self.panic_occurred = true;
        if self.is_at_end() {
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
        self.tokens[self.current].token_type == *token
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
