use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use crate::expression::{Binary, Expression, Grouping, Literal, Unary};
use crate::stmt::Stmt;
use crate::tokens::{LiteralTypes, Token, TokenType};

#[derive(Debug)]
pub struct InterpreterError {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}
impl Value {
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
        }
    }
}

pub struct Environment {

    // Parent environment for nested scopes
    enclosing: Option<Rc<RefCell<Environment>>>,

    // HashMap to store variable names and their values
    values: std::collections::HashMap<String, Value>,
}
impl Environment {
    pub fn new() -> Self {
        Environment { enclosing: None, values: std::collections::HashMap::new() }
    }

    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment { enclosing: Some(enclosing), values: std::collections::HashMap::new() }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<(), InterpreterError> {
        if self.values.contains_key(name.lexeme.as_str()) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(());
        }
        match &self.enclosing {
            Some(enclosing) => enclosing.borrow_mut().assign(name, value),
            None => Err(InterpreterError {
                message: format!("Undefined variable '{}'", name.lexeme),
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

pub struct Interpreter<'stmt> {
    // Environment for variable storage
    pub environment: Rc<RefCell<Environment>>,
    // Statements to be executed
    pub statements: &'stmt Vec<Stmt>,
    // Dedicated output stream for the interpreter
    pub output: Box<dyn Write>,
}

impl<'stmt> Interpreter<'stmt> {
    pub fn new(statements: &'stmt Vec<Stmt>) -> Self {
        let env = Rc::new(RefCell::new(Environment::new()));
        Interpreter { environment: env, statements, output: Box::new(std::io::stdout()) }
    }

    pub fn execute(&mut self) -> Result<(), InterpreterError> {
        for statement in self.statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, statement: &Stmt) -> Result<(), InterpreterError> {
        match statement {
            Stmt::Expression(expr_stmt) => {
                self.expression(&*expr_stmt.expression)?;
            }
            Stmt::Print(print_stmt) => {
                let value = self.expression(&*print_stmt.expression)?;
                writeln!(self.output, "{}", value);
            }
            Stmt::Block(block_stmt) => {
                self.execute_block(&block_stmt.statements)?;
            }
            Stmt::Var(var_stmt) => {
                if let Some(initializer) = &var_stmt.initializer {
                    let value = self.expression(&*initializer)?;
                    self.environment.borrow_mut().define(var_stmt.name.lexeme.clone(), value.clone());
                } else {
                    self.environment.borrow_mut().define(var_stmt.name.lexeme.clone(), Value::Nil);
                }
            }
        }
        Ok(())
    }

    fn execute_block(&mut self, statements: &Vec<Stmt>) -> Result<(), InterpreterError> {

        let previous = Rc::clone(&self.environment);
        let new_environment = Environment::with_enclosing(previous.clone());
        self.environment = Rc::new(RefCell::new(new_environment));

        for statement in statements {
            self.execute_statement(statement)?;
        }
        self.environment = previous;
        Ok(())
    }

    fn expression(&mut self, expression: &Expression) -> Result<Value, InterpreterError> {
        match expression {
            Expression::Binary(binary) => self.binary(binary),
            Expression::Grouping(grouping) => self.grouping(grouping),
            Expression::Literal(literal) => self.literal(literal),
            Expression::Unary(unary) => self.unary(unary),
            Expression::Variable(variable) => {
                match self.environment.borrow().get(&variable.name) {
                    Some(value) => Ok(value.clone()),
                    None => Err(InterpreterError {
                            message: format!("Variable {} not found", variable.name.lexeme),
                        })
                }
            },
            Expression::Assign(assign) => {
                let value = self.expression(&*assign.value)?;
                self.environment.borrow_mut().assign(&assign.name, value.clone());
                Ok(value)
            }
        }
    }

    fn grouping(&mut self, grouping: &Grouping) -> Result<Value, InterpreterError> {
        self.expression(&*grouping.expression)
    }

    fn literal(&self, literal: &Literal) -> Result<Value, InterpreterError> {
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
                _ => Err(InterpreterError {
                    message: "Operand must be a boolean".to_string(),
                }),
            },
            TokenType::Minus => match right {
                Value::Number(value) => Ok(Value::Number(-value)),
                _ => Err(InterpreterError {
                    message: "Operand must be a number".to_string(),
                }),
            },
            _ => Err(InterpreterError {
                message: "Invalid operator".to_string(),
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
                    message: "Operands must be numbers".to_string(),
                }),
            },
            TokenType::Slash => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left / right)),
                _ => Err(InterpreterError {
                    message: "Operands must be numbers".to_string(),
                }),
            },
            TokenType::Star => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left * right)),
                _ => Err(InterpreterError {
                    message: "Operands must be numbers".to_string(),
                }),
            },
            TokenType::Plus => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
                (Value::String(left), Value::String(right)) => Ok(Value::String(format!("{}{}", left, right))),
                _ => Err(InterpreterError {
                    message: "Operands must be numbers or strings".to_string(),
                }),
            },
            TokenType::Greater => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left > right)),
                _ => Err(InterpreterError {
                    message: "Operands must be numbers".to_string(),
                }),
            },
            TokenType::GreaterEqual => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left >= right)),
                _ => Err(InterpreterError {
                    message: "Operands must be numbers".to_string(),
                }),
            },
            TokenType::Less => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left < right)),
                _ => Err(InterpreterError {
                    message: "Operands must be numbers".to_string(),
                }),
            },
            TokenType::LessEqual => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left <= right)),
                _ => Err(InterpreterError {
                    message: "Operands must be numbers".to_string(),
                }),
            },
            TokenType::BangEqual => match (left, right) {
                (Value::Nil, Value::Nil) => Ok(Value::Bool(false)),
                (Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(left != right)),
                (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left != right)),
                (Value::String(left), Value::String(right)) => Ok(Value::Bool(left != right)),
                _ => Err(InterpreterError {
                    message: "Operands must be of the same type".to_string(),
                }),
            },
            TokenType::EqualEqual => match (left, right) {
                (Value::Nil, Value::Nil) => Ok(Value::Bool(true)),
                (Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(left == right)),
                (Value::Number(left), Value::Number(right)) => Ok(Value::Bool(left == right)),
                (Value::String(left), Value::String(right)) => Ok(Value::Bool(left == right)),
                _ => Err(InterpreterError {
                    message: "Operands must be of the same type".to_string(),
                }),
            },
            _ => Err(InterpreterError {
                message: "Invalid operator".to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::io::Write;
    use crate::{stmt::PrintStmt, tokens::Token};
    use crate::scanner::Scanner;
    use crate::parser::Parser;

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
        let mut interpreter = Interpreter { environment: Rc::new(RefCell::new(Environment::new())), statements: &statements, output: Box::new(VecWriter(Rc::clone(&output))) };
        let result = interpreter.execute();

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

        let statements: Vec<Stmt> = vec![];
        let mut interpreter: Interpreter<'_> = Interpreter::new(&statements);
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

        let statements: Vec<Stmt> = vec![];
        let mut interpreter: Interpreter<'_> = Interpreter::new(&statements);
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

        let statements: Vec<Stmt> = vec![];
        let mut interpreter: Interpreter<'_> = Interpreter::new(&statements);
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

        let statements: Vec<Stmt> = vec![];
        let mut interpreter: Interpreter<'_> = Interpreter::new(&statements);
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

        let statements: Vec<Stmt> = vec![];
        let mut interpreter: Interpreter<'_> = Interpreter::new(&statements);
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
        let mut interpreter = Interpreter { environment: Rc::new(RefCell::new(Environment::new())), statements: &statements, output: Box::new(VecWriter(Rc::clone(&output))) };
        interpreter.execute().unwrap();
        assert_eq!(String::from_utf8_lossy(&output.borrow()), "8\n");
    }

    #[test]
    fn test_print_multiple_expressions() {

        let source ="
        print \"one\";
        print true;
        print 2 + 1;
        ".to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "one\ntrue\n3\n");
    }

    #[test]
    fn test_uninitialized_variable() {
        let source = "
        var a;
        print a;
        ".to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "nil\n");
    }

    #[test]
    fn test_print_variable() {
        let source = "
        var a = 5;
        print a;
        ".to_string();

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
        ".to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "5\n10\n");
    }

    #[test]
    fn test_error_undefined_variable() {
        let source = "
        print a;
        ".to_string();

        let result = run(source);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "Variable a not found");
    }

    #[test]
    fn test_expression_from_variables() {
        let source = "
        var a = 5;
        var b = 3;
        print a + b;
        ".to_string();

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
        ".to_string();

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
        ".to_string();

        let result = run(source);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "Variable a not found");
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
        ".to_string();

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
        ".to_string();

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
        ".to_string();

        let result = run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "inner a\nouter b\nglobal c\nouter a\nouter b\nglobal c\nglobal a\nglobal b\nglobal c\n");
    }
}
