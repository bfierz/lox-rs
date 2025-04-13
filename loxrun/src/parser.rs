use crate::{
    expression::{Assign, Binary, Expression, Grouping, Literal, Logical, Unary, Variable},
    stmt::{BlockStmt, ExpressionStmt, IfStmt, PrintStmt, Stmt, VarStmt},
    tokens::{LiteralTypes, Token, TokenType}
};

// Production rules
// program -> statement* EOF ;

// declaration -> varDecl | statement ;
// varDecl -> "var" IDENTIFIER ("=" expression)? ";" ;
// statement -> exprStmt | ifStmt | printStmt | block ;
// exprStmt -> expression ";" ;
// ifStmt -> "if" "(" expression ")" statement ( "else" statement )? ;
// printStmt -> "print" expression ";" ;
// block -> "{" declaration* "}" ;

// expression -> assignment ;
// assignment -> IDENTIFIER "=" expression | logical_or ;
// logical_or -> logical_and ( "or" logical_and )* ;
// logical_and -> equality ( "and" equality )* ;
// equality -> comparison ( ( "!=" | "==" ) comparison )* ;
// comparison -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term -> factor ( ( "-" | "+" ) factor )* ;
// factor -> unary ( ( "/" | "*" ) unary )* ;
// unary -> ( "!" | "-" ) unary | primary ;
// primary -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER ;

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
        let mut statements = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    eprintln!("Error: {}", err.message);
                    self.synchronize();
                }
            }
        }
        Ok(statements)
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

    pub fn declaration(&mut self) -> Result<Stmt, ParserError> {
        if self.match_token(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }
    
    pub fn var_declaration(&mut self) -> Result<Stmt, ParserError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;
        let initializer = if self.match_token(&[TokenType::Equal]) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration.")?;
        Ok(Stmt::Var(VarStmt{name, initializer}))
    }

    pub fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.match_token(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_token(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_token(&[TokenType::LeftBrace]) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    pub fn if_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Stmt::If(IfStmt {
            condition: Box::new(condition),
            then_branch,
            else_branch,
        }))
    }

    pub fn print_statement(&mut self) -> Result<Stmt, ParserError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(PrintStmt {
            expression: Box::new(value),
        }))
    }

    pub fn block(&mut self) -> Result<Stmt, ParserError> {
        let mut statements = Vec::new();
        while !self.is_at_end() && self.tokens[self.current].token_type != TokenType::RightBrace {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    eprintln!("Error: {}", err.message);
                    self.synchronize();
                }
            }
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(Stmt::Block(BlockStmt { statements }))
    }

    pub fn expression_statement(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(ExpressionStmt {
            expression: Box::new(expr),
        }))
    }

    pub fn expression(&mut self) -> Result<Expression, ParserError> {
        self.assignment()
    }

    pub fn assignment(&mut self) -> Result<Expression, ParserError> {
        let expr = self.or()?;

        if self.match_token(&[TokenType::Equal]) {
            let value = self.assignment()?;
            match expr {
                Expression::Variable(ref var) => {
                    return Ok(Expression::Assign(Assign {
                        name: var.name.clone(),
                        value: Box::new(value),
                    }));
                }
                _ => {
                    return Err(ParserError { message: "Invalid assignment target".to_string() });
                }
            }
        }

        Ok(expr)
    }

    pub fn or(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.and()?;

        while self.match_token(&[TokenType::Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = Expression::Logical(Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    pub fn and(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.equality()?;

        while self.match_token(&[TokenType::And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expression::Logical(Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
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
        } else if self.match_token(&[TokenType::Identifier]) {
            let identifier = self.previous().clone();
            match identifier.literal {
                LiteralTypes::String(ref s) => {
                    if s.is_empty() {
                        return Err(ParserError { message: "Empty identifier".to_string() });
                    }
                    Ok(Expression::Variable(Variable {
                        name: identifier.clone(),
                    }))
                }
                _ => Err(ParserError { message: "Expected identifier".to_string() }),
            }
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
        let expression = "1 + 2 * 3 - 4 / 5;";

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
        let statements = &parser.parse().unwrap()[0];
        let expression = match statements {
            Stmt::Expression(ExpressionStmt { expression }) => expression.clone(),
            _ => panic!("Expected an expression statement"),
        };
        assert_eq!(*expression, reference);
    }
}