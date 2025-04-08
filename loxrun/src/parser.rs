use crate::{
    expression::{Expression, Binary, Grouping, Literal, Unary},
    tokens::{LiteralTypes, Token, TokenType},
    stmt::{Stmt, ExpressionStmt, PrintStmt},
};

// Production rules
// program -> statement* EOF ;

// statement -> exprStmt | printStmt ;
// exprStmt -> expression ";" ;
// printStmt -> "print" expression ";" ;

// expression -> equality ;
// equality -> comparison ( ( "!=" | "==" ) comparison )* ;
// comparison -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term -> factor ( ( "-" | "+" ) factor )* ;
// factor -> unary ( ( "/" | "*" ) unary )* ;
// unary -> ( "!" | "-" ) unary | primary ;
// primary -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug)]
pub struct ParserError {
    pub message: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut statements = Vec<Stmt>::new();
        while (!self.is_at_end()) {
            match self.statement() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    eprintln!("Error: {}", err.message);
                    self.synchronize();
                }
            }
        }
        statements
    }

    pub fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            // Check for valid tokens denoting the start of a new statement
            match self.tokens[self.current].token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => self.advance(),
            }
        }
    }

    pub fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.match_token(&[TokenType::Print]) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    pub fn print_statement(&mut self) -> Result<Stmt, ParserError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(PrintStmt {
            expression: Box::new(value),
        }))
    }

    pub fn expression_statement(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(ExpressionStmt {
            expression: Box::new(expr),
        }))
    }

    pub fn expression(&mut self) -> Result<Expression, ParserError> {
        self.equality()
    }

    pub fn equality(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expression::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    pub fn comparison(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.term()?;

        while self.match_token(&[TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expression::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    pub fn term(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expression::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    pub fn factor(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expression::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    pub fn unary(&mut self) -> Result<Expression, ParserError> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(Expression::Unary(Unary {
                operator,
                right: Box::new(right),
            }))
        } else {
            self.primary()
        }
    }

    pub fn primary(&mut self) -> Result<Expression, ParserError> {
        if self.match_token(&[TokenType::False]) {
            Ok(Expression::Literal(Literal {
                value: LiteralTypes::Bool(false),
            }))
        } else if self.match_token(&[TokenType::True]) {
            Ok(Expression::Literal(Literal {
                value: LiteralTypes::Bool(true),
            }))
        } else if self.match_token(&[TokenType::Nil]) {
            Ok(Expression::Literal(Literal {
                value: LiteralTypes::Nil,
            }))
        } else if self.match_token(&[TokenType::Number]) {
            let number = self.previous().clone();
            Ok(Expression::Literal(Literal {
                value: number.literal,
            }))
        } else if self.match_token(&[TokenType::String]) {
            let string = self.previous().clone();
            Ok(Expression::Literal(Literal {
                value: string.literal,
            }))
        } else if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            Ok(Expression::Grouping(Grouping {
                expression: Box::new(expr),
            }))
        } else {
            Err(ParserError {message: "Expected literal or grouping".to_string()})
        }
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
            Err(ParserError { message: message.to_string() })
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;

    #[test]
    fn test_parser() {
        let expression = "1 + 2 * 3 - 4 / 5";

        let four_div_five = Box::new(Expression::Binary(Binary {
            left: Box::new(Expression::Literal(Literal {
                value: LiteralTypes::Number(4.0),
            })),
            operator: Token {
                token_type: TokenType::Slash,
                lexeme: "/".to_string(),
                literal: LiteralTypes::Nil,
                line: 1,
            },
            right: Box::new(Expression::Literal(Literal {
                value: LiteralTypes::Number(5.0),
            })),
        }));
        let two_mul_three = Box::new(Expression::Binary(Binary {
            left: Box::new(Expression::Literal(Literal {
                value: LiteralTypes::Number(2.0),
            })),
            operator: Token {
                token_type: TokenType::Star,
                lexeme: "*".to_string(),
                literal: LiteralTypes::Nil,
                line: 1,
            },
            right: Box::new(Expression::Literal(Literal {
                value: LiteralTypes::Number(3.0),
            })),
        }));
        let reference = Expression::Binary(Binary {
            left: Box::new(Expression::Binary(Binary {
                left: Box::new(Expression::Literal(Literal {
                    value: LiteralTypes::Number(1.0),
                })),
                operator: Token {
                    token_type: TokenType::Plus,
                    lexeme: "+".to_string(),
                    literal: LiteralTypes::Nil,
                    line: 1,
                },
                right: two_mul_three,
            })),
            operator: Token {
                token_type: TokenType::Minus,
                lexeme: "-".to_string(),
                literal: LiteralTypes::Nil,
                line: 1,
            },
            right: four_div_five,
        });

        let mut scanner = Scanner::new(expression.to_string());
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens.clone());
        let expression = parser.parse().unwrap();
        assert_eq!(expression, reference);
    }
}