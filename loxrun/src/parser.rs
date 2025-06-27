use crate::{
    expression::{
        Assign, Binary, Call, Expression, Get, Grouping, Literal, Logical, Set, This, Unary,
        Variable,
    },
    stmt::{
        BlockStmt, ClassStmt, ExpressionStmt, FunctionStmt, IfStmt, PrintStmt, ReturnStmt, Stmt,
        VarStmt, WhileStmt,
    },
    tokens::{LiteralTypes, Token, TokenType},
};

// Production rules
// program -> statement* EOF ;

// declaration -> classDecl | funDecl | varDecl | statement ;
// classDecl -> "class" IDENTIFIER "{" function* "}" ;
// funDecls -> "fun" function ;
// function -> IDENTIFIER "(" parameters? ")" block ;
// parameters -> IDENTIFIER ( "," IDENTIFIER )* ;
// varDecl -> "var" IDENTIFIER ("=" expression)? ";" ;
// statement -> exprStmt | forStmt | ifStmt | printStmt | returnStm | whileStmt | block ;
// exprStmt -> expression ";" ;
// forStmt -> "for" "(" (varDecl | exprStmt | ";") expression? ";" expression? ")" statement ;
// ifStmt -> "if" "(" expression ")" statement ( "else" statement )? ;
// printStmt -> "print" expression ";" ;
// returnStmt -> "return" expression? ";" ;
// whileStmt -> "while" "(" expression ")" statement ;
// block -> "{" declaration* "}" ;

// expression -> assignment ;
// assignment -> ( call "." )? IDENTIFIER "=" assignment | logical_or ;
// logical_or -> logical_and ( "or" logical_and )* ;
// logical_and -> equality ( "and" equality )* ;
// equality -> comparison ( ( "!=" | "==" ) comparison )* ;
// comparison -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term -> factor ( ( "-" | "+" ) factor )* ;
// factor -> unary ( ( "/" | "*" ) unary )* ;
// unary -> ( "!" | "-" ) unary | call ;
// call -> primary ( "(" arguments? ")" )* ;
// primary -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER ;
// arguments -> expression ( "," expression )* ;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    current_id: usize,
}

#[derive(Debug)]
pub struct ParserError {
    pub message: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            current_id: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut has_error = false;
        let mut statements = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    has_error = true;
                    eprintln!("{}", err.message);
                    self.synchronize();
                }
            }
        }
        if has_error {
            return Err(ParserError {
                message: "Parsing failed with errors.".to_string(),
            });
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
        if self.match_token(&[TokenType::Class]) {
            self.class_declaration()
        } else if self.match_token(&[TokenType::Fun]) {
            self.fun_declaration("function".to_string())
        } else if self.match_token(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    pub fn class_declaration(&mut self) -> Result<Stmt, ParserError> {
        let name = self.consume(TokenType::Identifier, "Expect class name.")?;
        self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;

        let mut methods = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let stmt = self.fun_declaration("method".to_string())?;
            if let Stmt::Function(method) = stmt {
                methods.push(method);
            }
        }
        self.consume(TokenType::RightBrace, "Expect '}' after class body.")?;
        Ok(Stmt::Class(ClassStmt { name, methods }))
    }

    pub fn fun_declaration(&mut self, kind: String) -> Result<Stmt, ParserError> {
        let name = self.consume_msg(TokenType::Identifier, format!("Expect {} name.", kind))?;
        self.consume_msg(
            TokenType::LeftParen,
            format!("Expect '(' after {} name.", kind),
        )?;

        let mut params = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    let line = self.tokens[self.current].line;
                    let name = &self.tokens[self.current].lexeme;
                    return Err(ParserError {
                        message: format!(
                            "[line {}] Error at '{}': {}",
                            line, name, "Can't have more than 255 parameters."
                        ),
                    });
                }
                params.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;
        self.consume(TokenType::LeftBrace, "Expect '{' before function body.")?;
        let body = match self.block()? {
            Stmt::Block(block) => block,
            _ => {
                return Err(ParserError {
                    message: "Expected block after function declaration.".to_string(),
                })
            }
        };
        Ok(Stmt::Function(FunctionStmt {
            name,
            params,
            body: body.statements,
        }))
    }

    pub fn var_declaration(&mut self) -> Result<Stmt, ParserError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;
        let initializer = if self.match_token(&[TokenType::Equal]) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var(VarStmt { name, initializer }))
    }

    pub fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.match_token(&[TokenType::For]) {
            self.for_statement()
        } else if self.match_token(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_token(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_token(&[TokenType::Return]) {
            self.return_statement()
        } else if self.match_token(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_token(&[TokenType::LeftBrace]) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    pub fn for_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.match_token(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else if self.match_token(&[TokenType::Semicolon]) {
            None
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(&TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if !self.check(&TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = Box::new(self.statement()?);

        if let Some(increment) = increment {
            body = Box::new(Stmt::Block(BlockStmt {
                statements: vec![
                    *body,
                    Stmt::Expression(ExpressionStmt {
                        expression: Box::new(increment),
                    }),
                ],
            }));
        }

        if let Some(condition) = condition {
            body = Box::new(Stmt::While(WhileStmt {
                condition: Box::new(condition),
                body,
            }));
        } else {
            body = Box::new(Stmt::While(WhileStmt {
                condition: Box::new(Expression::Literal(Literal {
                    id: self.next_id(),
                    value: LiteralTypes::Bool(true),
                })),
                body,
            }));
        }

        if let Some(initializer) = initializer {
            Ok(Stmt::Block(BlockStmt {
                statements: vec![initializer, *body],
            }))
        } else {
            Ok(*body)
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

    pub fn return_statement(&mut self) -> Result<Stmt, ParserError> {
        let keyword = self.previous().clone();
        let value = if !self.check(&TokenType::Semicolon) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
        Ok(Stmt::Return(ReturnStmt { keyword, value }))
    }

    pub fn while_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::While(WhileStmt {
            condition: Box::new(condition),
            body,
        }))
    }

    pub fn block(&mut self) -> Result<Stmt, ParserError> {
        let mut has_error = false;
        let mut last_error: String = "".to_string();
        let mut statements = Vec::new();
        while !self.is_at_end() && self.tokens[self.current].token_type != TokenType::RightBrace {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    has_error = true;
                    last_error = err.message.clone();
                    self.synchronize();
                }
            }
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        if has_error {
            return Err(ParserError {
                message: last_error,
            });
        }
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
                        id: self.next_id(),
                        name: var.name.clone(),
                        value: Box::new(value),
                    }));
                }
                Expression::Get(ref get) => {
                    return Ok(Expression::Set(Set {
                        id: self.next_id(),
                        object: get.object.clone(),
                        name: get.name.clone(),
                        value: Box::new(value),
                    }));
                }
                _ => {
                    return Err(ParserError {
                        message: format!(
                            "[line {}] Error at '=': Invalid assignment target.",
                            self.previous().line
                        ),
                    });
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
                id: self.next_id(),
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
                id: self.next_id(),
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
                id: self.next_id(),
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    pub fn comparison(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expression::Binary(Binary {
                id: self.next_id(),
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
                id: self.next_id(),
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
                id: self.next_id(),
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
                id: self.next_id(),
                operator,
                right: Box::new(right),
            }))
        } else {
            self.call()
        }
    }

    pub fn call(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenType::Dot]) {
                let name =
                    self.consume(TokenType::Identifier, "Expect property name after '.'.")?;
                expr = Expression::Get(Get {
                    id: self.next_id(),
                    object: Box::new(expr),
                    name,
                });
            } else {
                break;
            }
        }

        Ok(expr)
    }

    pub fn finish_call(&mut self, callee: Expression) -> Result<Expression, ParserError> {
        let mut arguments = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    let line = self.tokens[self.current].line;
                    let name = &self.tokens[self.current].lexeme;
                    return Err(ParserError {
                        message: format!(
                            "[line {}] Error at '{}': {}",
                            line, name, "Can't have more than 255 arguments."
                        ),
                    });
                }
                arguments.push(self.expression()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
        Ok(Expression::Call(Call {
            id: self.next_id(),
            callee: Box::new(callee),
            paren,
            arguments,
        }))
    }

    pub fn primary(&mut self) -> Result<Expression, ParserError> {
        if self.match_token(&[TokenType::False]) {
            Ok(Expression::Literal(Literal {
                id: self.next_id(),
                value: LiteralTypes::Bool(false),
            }))
        } else if self.match_token(&[TokenType::True]) {
            Ok(Expression::Literal(Literal {
                id: self.next_id(),
                value: LiteralTypes::Bool(true),
            }))
        } else if self.match_token(&[TokenType::Nil]) {
            Ok(Expression::Literal(Literal {
                id: self.next_id(),
                value: LiteralTypes::Nil,
            }))
        } else if self.match_token(&[TokenType::Number]) {
            let number = self.previous().clone();
            Ok(Expression::Literal(Literal {
                id: self.next_id(),
                value: number.literal,
            }))
        } else if self.match_token(&[TokenType::String]) {
            let string = self.previous().clone();
            Ok(Expression::Literal(Literal {
                id: self.next_id(),
                value: string.literal,
            }))
        } else if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            Ok(Expression::Grouping(Grouping {
                id: self.next_id(),
                expression: Box::new(expr),
            }))
        } else if self.match_token(&[TokenType::This]) {
            Ok(Expression::This(This {
                id: self.next_id(),
                keyword: self.previous().clone(),
            }))
        } else if self.match_token(&[TokenType::Identifier]) {
            let identifier = self.previous().clone();
            match identifier.literal {
                LiteralTypes::String(ref s) => {
                    if s.is_empty() {
                        return Err(ParserError {
                            message: "Empty identifier".to_string(),
                        });
                    }
                    Ok(Expression::Variable(Variable {
                        id: self.next_id(),
                        name: identifier.clone(),
                    }))
                }
                _ => Err(ParserError {
                    message: "Expected identifier".to_string(),
                }),
            }
        } else {
            let line = self.tokens[self.current].line;
            let name = self.tokens[self.current].lexeme.clone();
            Err(ParserError {
                message: format!(
                    "[line {}] Error at '{}': {}",
                    line, name, "Expect expression."
                ),
            })
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
        self.consume_msg(token, message.to_string())
    }

    pub fn consume_msg(&mut self, token: TokenType, message: String) -> Result<Token, ParserError> {
        if self.check(&token) {
            self.advance();
            Ok(self.previous())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;

    #[test]
    fn test_parser() {
        let expression = "1 + 2 * 3 - 4 / 5;";

        let four_div_five = Box::new(Expression::Binary(Binary {
            id: 7,
            left: Box::new(Expression::Literal(Literal {
                id: 5,
                value: LiteralTypes::Number(4.0),
            })),
            operator: Token {
                token_type: TokenType::Slash,
                lexeme: "/".to_string(),
                literal: LiteralTypes::Nil,
                line: 1,
            },
            right: Box::new(Expression::Literal(Literal {
                id: 6,
                value: LiteralTypes::Number(5.0),
            })),
        }));
        let two_mul_three = Box::new(Expression::Binary(Binary {
            id: 3,
            left: Box::new(Expression::Literal(Literal {
                id: 1,
                value: LiteralTypes::Number(2.0),
            })),
            operator: Token {
                token_type: TokenType::Star,
                lexeme: "*".to_string(),
                literal: LiteralTypes::Nil,
                line: 1,
            },
            right: Box::new(Expression::Literal(Literal {
                id: 2,
                value: LiteralTypes::Number(3.0),
            })),
        }));
        let reference = Expression::Binary(Binary {
            id: 8,
            left: Box::new(Expression::Binary(Binary {
                id: 4,
                left: Box::new(Expression::Literal(Literal {
                    id: 0,
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
