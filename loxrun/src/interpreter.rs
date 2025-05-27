use crate::callable::{Callable, LoxCallable, LoxFunction};
use crate::expression::{Binary, Call, Expression, Grouping, Literal, Logical, Unary};
use crate::stmt::Stmt;
use crate::tokens::{LiteralTypes, Token, TokenType};
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

#[derive(Debug)]
pub struct InterpreterError {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Callable(Callable),
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}
impl Value {
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    pub fn is_true(&self) -> bool {
        match self {
            Value::Bool(value) => *value,
            Value::Nil => false,
            _ => true,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Callable(_) => write!(f, "<function>"),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
        }
    }
}

pub enum InterpreterResult {
    None,
    Return(Value),
}

#[derive(Debug, PartialEq)]
pub struct Environment {
    // Parent environment for nested scopes
    enclosing: Option<Rc<RefCell<Environment>>>,

    // HashMap to store variable names and their values
    values: std::collections::HashMap<String, Value>,
}
impl Environment {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: std::collections::HashMap::new(),
        }
    }

    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment {
            enclosing: Some(enclosing),
            values: std::collections::HashMap::new(),
        }
    }

    pub fn deep_clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            enclosing: self
                .enclosing
                .as_ref()
                .map(|env| std::rc::Rc::new(std::cell::RefCell::new(env.borrow().deep_clone()))),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn assign(
        &mut self,
        name: &Token,
        value: Value,
    ) -> Result<InterpreterResult, InterpreterError> {
        if self.values.contains_key(name.lexeme.as_str()) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(InterpreterResult::None);
        }
        match &self.enclosing {
            Some(enclosing) => enclosing.borrow_mut().assign(name, value),
            None => Err(InterpreterError {
                message: format!(
                    "Undefined variable '{}'.\n[line {}]",
                    name.lexeme, name.line
                ),
            }),
        }
    }

    pub fn get(&self, name: &Token) -> Option<Value> {
        let result = self.values.get(name.lexeme.as_str());

        if result.is_some() {
            return Some(result.unwrap().clone());
        }
        match &self.enclosing {
            Some(enclosing) => enclosing.as_ref().borrow().get(name),
            None => None,
        }
    }
}

pub struct Interpreter {
    // Global environment for variable storage
    pub globals: Rc<RefCell<Environment>>,
    // Environment for variable storage
    pub environment: Rc<RefCell<Environment>>,
    // Dedicated output stream for the interpreter
    pub output: Box<dyn Write>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));
        //globals.borrow_mut().define(
        //    "clock".to_string(),
        //    Value::Callable(LoxBuiltinFunctionClock::new()),
        //);
        Interpreter {
            globals: Rc::clone(&globals),
            environment: globals,
            output: Box::new(std::io::stdout()),
        }
    }

    pub fn execute(
        &mut self,
        statements: &Vec<Stmt>,
    ) -> Result<InterpreterResult, InterpreterError> {
        for statement in statements {
            self.execute_statement(statement)?;
        }
        Ok(InterpreterResult::None)
    }

    fn execute_statement(
        &mut self,
        statement: &Stmt,
    ) -> Result<InterpreterResult, InterpreterError> {
        match statement {
            Stmt::Expression(expr_stmt) => {
                self.expression(&*expr_stmt.expression)?;
            }
            Stmt::Function(fun_stmt) => {
                self.environment.borrow_mut().define(
                    fun_stmt.name.lexeme.clone(),
                    Value::Callable(Callable::Function(LoxFunction::new(
                        fun_stmt.clone(),
                        self.environment.clone(),
                    ))),
                );
            }
            Stmt::Return(return_stmt) => {
                if let Some(value) = &return_stmt.value {
                    let return_value = self.expression(&*value)?;
                    return Ok(InterpreterResult::Return(return_value));
                } else {
                    return Ok(InterpreterResult::Return(Value::Nil));
                }
            }
            Stmt::If(if_stmt) => {
                let condition = self.expression(&*if_stmt.condition)?;
                if condition.is_true() {
                    return self.execute_statement(&*if_stmt.then_branch);
                } else if let Some(else_branch) = &if_stmt.else_branch {
                    return self.execute_statement(else_branch);
                }
            }
            Stmt::Print(print_stmt) => {
                let value = self.expression(&*print_stmt.expression)?;
                writeln!(self.output, "{}", value);
            }
            Stmt::Block(block_stmt) => {
                return self.execute_block(&block_stmt.statements, self.environment.clone());
            }
            Stmt::Var(var_stmt) => {
                if let Some(initializer) = &var_stmt.initializer {
                    let value = self.expression(&*initializer)?;
                    self.environment
                        .borrow_mut()
                        .define(var_stmt.name.lexeme.clone(), value.clone());
                } else {
                    self.environment
                        .borrow_mut()
                        .define(var_stmt.name.lexeme.clone(), Value::Nil);
                }
            }
            Stmt::While(while_stmt) => {
                while self.expression(&*while_stmt.condition)?.is_true() {
                    if let Ok(InterpreterResult::Return(value)) =
                        self.execute_statement(&*while_stmt.body)
                    {
                        return Ok(InterpreterResult::Return(value));
                    }
                }
            }
        }
        Ok(InterpreterResult::None)
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<InterpreterResult, InterpreterError> {
        let previous = Rc::clone(&self.environment);
        let new_environment = Environment::with_enclosing(environment);
        self.environment = Rc::new(RefCell::new(new_environment));

        let mut result = InterpreterResult::None;
        for statement in statements {
            if let Ok(InterpreterResult::Return(value)) = self.execute_statement(statement) {
                result = InterpreterResult::Return(value);
                break;
            }
        }
        self.environment = previous;
        Ok(result)
    }

    fn expression(&mut self, expression: &Expression) -> Result<Value, InterpreterError> {
        match expression {
            Expression::Binary(binary) => self.binary(binary),
            Expression::Call(call) => self.call(call),
            Expression::Grouping(grouping) => self.grouping(grouping),
            Expression::Literal(literal) => self.literal(literal),
            Expression::Logical(logical) => self.logical(logical),
            Expression::Unary(unary) => self.unary(unary),
            Expression::Variable(variable) => match self.environment.borrow().get(&variable.name) {
                Some(value) => Ok(value.clone()),
                None => Err(InterpreterError {
                    message: format!(
                        "Undefined variable '{}'.\n[line {}]",
                        variable.name.lexeme, variable.name.line
                    ),
                }),
            },
            Expression::Assign(assign) => {
                let value = self.expression(&*assign.value)?;
                self.environment
                    .borrow_mut()
                    .assign(&assign.name, value.clone())?;
                Ok(value)
            }
        }
    }

    fn call(&mut self, call: &Call) -> Result<Value, InterpreterError> {
        let callee = self.expression(&*call.callee)?;
        if let Value::Callable(callable) = &callee {
            if let Callable::Function(func) = callable {
                let mut arguments = Vec::new();
                for arg in &call.arguments {
                    arguments.push(self.expression(arg)?);
                }
                if arguments.len() != func.arity() {
                    return Err(InterpreterError {
                        message: format!(
                            "Expected {} arguments but got {}",
                            func.arity(),
                            arguments.len()
                        ),
                    });
                }
                func.call(self, arguments)
            } else {
                return Err(InterpreterError {
                    message: "Can only call functions".to_string(),
                });
            }
        } else {
            return Err(InterpreterError {
                message: format!(
                    "Can only call functions and classes.\n[line {}]",
                    call.paren.line
                ),
            });
        }
    }

    fn grouping(&mut self, grouping: &Grouping) -> Result<Value, InterpreterError> {
        self.expression(&*grouping.expression)
    }

    fn logical(&mut self, logical: &Logical) -> Result<Value, InterpreterError> {
        let left = self.expression(&*logical.left)?;
        if logical.operator.token_type == TokenType::Or {
            if left.is_true() {
                return Ok(left);
            }
        } else {
            if !left.is_true() {
                return Ok(left);
            }
        }
        self.expression(&*logical.right)
    }

    pub fn literal(&self, literal: &Literal) -> Result<Value, InterpreterError> {
        match &literal.value {
            LiteralTypes::String(value) => Ok(Value::String(value.clone())),
            LiteralTypes::Number(value) => Ok(Value::Number(*value)),
            LiteralTypes::Bool(value) => Ok(Value::Bool(*value)),
            LiteralTypes::Nil => Ok(Value::Nil),
        }
    }

    fn unary(&mut self, unary: &Unary) -> Result<Value, InterpreterError> {
        let right = self.expression(&*unary.right)?;

        match unary.operator.token_type {
            TokenType::Bang => match right {
                Value::Bool(value) => Ok(Value::Bool(!value)),
                Value::Nil => Ok(Value::Bool(true)),
                _ => Ok(Value::Bool(false)),
            },
            TokenType::Minus => match right {
                Value::Number(value) => Ok(Value::Number(-value)),
                _ => Err(InterpreterError {
                    message: format!("Operand must be a number.\n[line {}]", unary.operator.line),
                }),
            },
            _ => Err(InterpreterError {
                message: format!(
                    "Invalid operator '{}'.\n[line {}]",
                    unary.operator.lexeme, unary.operator.line
                ),
            }),
        }
    }

    fn binary(&mut self, binary: &Binary) -> Result<Value, InterpreterError> {
        let left = self.expression(&*binary.left)?;
        let right = self.expression(&*binary.right)?;

        match binary.operator.token_type {
            TokenType::Minus => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left - right)),
                _ => Err(InterpreterError {
                    message: format!("Operands must be numbers.\n[line {}]", binary.operator.line),
                }),
            },
            TokenType::Slash => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left / right)),
                _ => Err(InterpreterError {
                    message: format!("Operands must be numbers.\n[line {}]", binary.operator.line),
                }),
            },
            TokenType::Star => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left * right)),
                _ => Err(InterpreterError {
                    message: format!("Operands must be numbers.\n[line {}]", binary.operator.line),
                }),
            },
            TokenType::Plus => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
                (Value::String(left), Value::String(right)) => {
                    Ok(Value::String(format!("{}{}", left, right)))
                }
                _ => Err(InterpreterError {
                    message: format!(
                        "Operands must be two numbers or two strings.\n[line {}]",
                        binary.operator.line
                    ),
                }),
            },
            TokenType::Greater => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left > right)),
                _ => Err(InterpreterError {
                    message: format!("Operands must be numbers.\n[line {}]", binary.operator.line),
                }),
            },
            TokenType::GreaterEqual => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left >= right)),
                _ => Err(InterpreterError {
                    message: format!("Operands must be numbers.\n[line {}]", binary.operator.line),
                }),
            },
            TokenType::Less => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left < right)),
                _ => Err(InterpreterError {
                    message: format!("Operands must be numbers.\n[line {}]", binary.operator.line),
                }),
            },
            TokenType::LessEqual => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left <= right)),
                _ => Err(InterpreterError {
                    message: format!("Operands must be numbers.\n[line {}]", binary.operator.line),
                }),
            },
            TokenType::BangEqual => match (left, right) {
                (Value::Nil, Value::Nil) => Ok(Value::Bool(false)),
                (Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(left != right)),
                (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left != right)),
                (Value::String(left), Value::String(right)) => Ok(Value::Bool(left != right)),
                _ => Ok(Value::Bool(true)),
            },
            TokenType::EqualEqual => match (left, right) {
                (Value::Nil, Value::Nil) => Ok(Value::Bool(true)),
                (Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(left == right)),
                (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left == right)),
                (Value::String(left), Value::String(right)) => Ok(Value::Bool(left == right)),
                _ => Ok(Value::Bool(false)),
            },
            _ => Err(InterpreterError {
                message: "Invalid operator.".to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::scanner::Scanner;
    use crate::{stmt::PrintStmt, tokens::Token};
    use std::io;
    use std::io::Write;

    // Mocking the output stream for testing
    struct VecWriter(Rc<RefCell<Vec<u8>>>);

    impl Write for VecWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn run(source: String) -> Result<String, InterpreterError> {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().clone();
        assert!(!scanner.had_error);

        let mut parser = Parser::new(tokens);
        let parse_result = parser.parse();
        assert!(parse_result.is_ok());

        let statements = parse_result.unwrap();

        let output = Rc::new(RefCell::new(Vec::<u8>::new()));
        let globals = Rc::new(RefCell::new(Environment::new()));
        let mut interpreter = Interpreter {
            globals: Rc::clone(&globals),
            environment: globals,
            output: Box::new(VecWriter(Rc::clone(&output))),
        };
        let result = interpreter.execute(&statements);

        match result {
            Ok(_) => Ok(String::from_utf8_lossy(&output.borrow()).to_string()),
            Err(err) => Err(err),
        }
    }

    #[test]
    fn test_interpret_sum() {
        let expression = Expression::Binary(Binary {
            left: Box::new(Expression::Literal(Literal {
                value: LiteralTypes::Number(5.0),
            })),
            operator: Token {
                token_type: TokenType::Plus,
                lexeme: "+".to_string(),
                literal: LiteralTypes::Nil,
                line: 1,
            },
            right: Box::new(Expression::Literal(Literal {
                value: LiteralTypes::Number(3.0),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.expression(&expression).unwrap();
        assert_eq!(result, Value::Number(8.0));
    }

    #[test]
    fn test_interpret_subtraction() {
        let expression = Expression::Binary(Binary {
            left: Box::new(Expression::Literal(Literal {
                value: LiteralTypes::Number(5.0),
            })),
            operator: Token {
                token_type: TokenType::Minus,
                lexeme: "-".to_string(),
                literal: LiteralTypes::Nil,
                line: 1,
            },
            right: Box::new(Expression::Literal(Literal {
                value: LiteralTypes::Number(3.0),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.expression(&expression).unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_interpret_multiplication() {
        let expression = Expression::Binary(Binary {
            left: Box::new(Expression::Literal(Literal {
                value: LiteralTypes::Number(5.0),
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
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.expression(&expression).unwrap();
        assert_eq!(result, Value::Number(15.0));
    }
    #[test]
    fn test_interpret_division() {
        let expression = Expression::Binary(Binary {
            left: Box::new(Expression::Literal(Literal {
                value: LiteralTypes::Number(6.0),
            })),
            operator: Token {
                token_type: TokenType::Slash,
                lexeme: "/".to_string(),
                literal: LiteralTypes::Nil,
                line: 1,
            },
            right: Box::new(Expression::Literal(Literal {
                value: LiteralTypes::Number(3.0),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.expression(&expression).unwrap();
        assert_eq!(result, Value::Number(2.0));
    }
    #[test]
    fn test_star_before_plus() {
        let expression = Expression::Binary(Binary {
            left: Box::new(Expression::Binary(Binary {
                left: Box::new(Expression::Literal(Literal {
                    value: LiteralTypes::Number(5.0),
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
            })),
            operator: Token {
                token_type: TokenType::Plus,
                lexeme: "+".to_string(),
                literal: LiteralTypes::Nil,
                line: 1,
            },
            right: Box::new(Expression::Literal(Literal {
                value: LiteralTypes::Number(2.0),
            })),
        });

        let mut interpreter = Interpreter::new();
        let result = interpreter.expression(&expression).unwrap();
        assert_eq!(result, Value::Number(17.0));
    }

    #[test]
    fn test_print_expression() {
        let expression = Expression::Binary(Binary {
            left: Box::new(Expression::Literal(Literal {
                value: LiteralTypes::Number(5.0),
            })),
            operator: Token {
                token_type: TokenType::Plus,
                lexeme: "+".to_string(),
                literal: LiteralTypes::Nil,
                line: 1,
            },
            right: Box::new(Expression::Literal(Literal {
                value: LiteralTypes::Number(3.0),
            })),
        });

        let print_stmt = Stmt::Print(PrintStmt {
            expression: Box::new(expression),
        });
        let statements = vec![print_stmt];
        let output = Rc::new(RefCell::new(Vec::<u8>::new()));
        let globals = Rc::new(RefCell::new(Environment::new()));
        let mut interpreter = Interpreter {
            globals: Rc::clone(&globals),
            environment: globals,
            output: Box::new(VecWriter(Rc::clone(&output))),
        };
        interpreter.execute(&statements).unwrap();
        assert_eq!(String::from_utf8_lossy(&output.borrow()), "8\n");
    }

    #[test]
    fn test_print_multiple_expressions() {
        let source = "
        print \"one\";
        print true;
        print 2 + 1;
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "one\ntrue\n3\n");
    }

    #[test]
    fn test_uninitialized_variable() {
        let source = "
        var a;
        print a;
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "nil\n");
    }

    #[test]
    fn test_print_variable() {
        let source = "
        var a = 5;
        print a;
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "5\n");
    }

    #[test]
    fn print_redefined_variable() {
        let source = "
        var a = 5;
        print a;
        var a = 10;
        print a;
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "5\n10\n");
    }

    #[test]
    fn test_error_undefined_variable() {
        let source = "
        print a;
        "
        .to_string();

        let result = run(source);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message,
            "Undefined variable 'a'.\n[line 2]"
        );
    }

    #[test]
    fn test_expression_from_variables() {
        let source = "
        var a = 5;
        var b = 3;
        print a + b;
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "8\n");
    }

    #[test]
    fn test_assignment() {
        let source = "
        var a = 5;
        print a;
        a = 10;
        print a;
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "5\n10\n");
    }

    #[test]
    fn test_variable_used_outside_scope() {
        let source = "
        {
            var a = 5;
            print a;
        }
        print a;
        "
        .to_string();

        let result = run(source);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message,
            "Undefined variable 'a'.\n[line 6]"
        );
    }

    #[test]
    fn test_variable_shadowing() {
        let source = "
        var a = 5;
        {
            var a = 10;
            print a;
        }
        print a;
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "10\n5\n");
    }

    #[test]
    fn test_variables_from_inner_scope() {
        let source = "
        var a = 5;
        {
            var b = 10;
            print a + b;
        }
        print a;
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "15\n5\n");
    }

    #[test]
    fn test_variables_from_three_scopes() {
        let source = "
        var a = \"global a\";
        var b = \"global b\";
        var c = \"global c\";
        {
            var a = \"outer a\";
            var b = \"outer b\";
            {
                var a = \"inner a\";
                print a;
                print b;
                print c;
            }
            print a;
            print b;
            print c;
        }
        print a;
        print b;
        print c;
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "inner a\nouter b\nglobal c\nouter a\nouter b\nglobal c\nglobal a\nglobal b\nglobal c\n");
    }

    #[test]
    fn test_if_statement_true() {
        let source = "
        if (true) {
            print \"True\";
        } else {
            print \"False\";
        }
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "True\n");
    }

    #[test]
    fn test_if_statement_false() {
        let source = "
        if (false) {
            print \"True\";
        } else {
            print \"False\";
        }
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "False\n");
    }

    #[test]
    fn test_if_statement_expression() {
        let source = "
        if (3 < 2) {
            print \"True\";
        } else {
            print \"False\";
        }
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "False\n");
    }

    #[test]
    fn test_if_statement_zero_is_true() {
        let source = "
        if (0) {
            print \"True\";
        } else {
            print \"False\";
        }
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "True\n");
    }

    #[test]
    fn test_logical_or() {
        let source = "
        print true or false;
        print false or true;
        print false or false;
        print true or true;
        print 0 or 1;
        print 0 or false;
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "true\ntrue\nfalse\ntrue\n0\n0\n");
    }

    #[test]
    fn test_while_statement() {
        let source = "
        var i = 0;
        while (i < 5) {
            print i;
            i = i + 1;
        }
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0\n1\n2\n3\n4\n");
    }

    #[test]
    fn test_for_statement() {
        let source = "
        for (var i = 0; i < 5; i = i + 1) {
            print i;
        }
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0\n1\n2\n3\n4\n");
    }

    #[test]
    fn test_function_definition_and_call() {
        let source = "
        fun greet() {
            print \"Hello, World!\";
        }
        greet();
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!\n");
    }

    #[test]
    fn test_function_definition_and_call_with_param() {
        let source = "
        fun greet(name) {
            print \"Hello, \" + name + \"!\";
        }
        greet(\"World\");
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!\n");
    }

    #[test]
    fn test_function_definition_and_call_with_return() {
        let source = "
        fun greet() {
            return \"Hello, World!\";
        }
        print greet();
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!\n");
    }

    #[test]
    fn test_function_definition_and_call_with_multiple_params() {
        let source = "
        fun add(a, b) {
            return a + b;
        }
        print add(5, 3);
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "8\n");
    }

    #[test]
    fn test_recursion() {
        let source = "
        fun factorial(n) {
            if (n == 0) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        print factorial(5);
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "120\n");
    }

    #[test]
    fn test_fibonacci() {
        let source = "
        fun fib(n) {
            if (n <= 1) return n;
            return fib(n - 2) + fib(n - 1);
        }
        print fib(8);
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "21\n");
    }

    #[test]
    fn test_function_object_with_closure() {
        let source = "
        fun makeCounter() {
            var i = 0;
            fun count() {
                i = i + 1;
                return i;
            }
            return count;
        }
        var counter = makeCounter();
        print counter();
        print counter();
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1\n2\n");
    }

    #[test]
    fn test_function_object_with_closure_and_outer_variable() {
        let source = "
        var outerVar = 10;
        fun makeCounter() {
            var i = 0;
            fun count() {
                i = i + 1;
                return i + outerVar;
            }
            return count;
        }
        var counter = makeCounter();
        print counter();
        print counter();
        "
        .to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "11\n12\n");
    }
}
