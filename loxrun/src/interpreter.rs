use crate::expression::{Binary, Expression, Grouping, Literal, Unary};
use crate::tokens::{LiteralTypes, TokenType};

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

pub struct Interpreter<'expr> {
    pub expression: &'expr Expression,
}

impl<'expr> Interpreter<'expr> {
    pub fn new(expression: &'expr Expression) -> Self {
        Interpreter { expression }
    }

    pub fn evaluate(&self) -> Result<Value, InterpreterError> {
        self.expression(&self.expression)
    }

    fn expression(&self, expression: &Expression) -> Result<Value, InterpreterError> {
        match expression {
            Expression::Binary(binary) => self.binary(binary),
            Expression::Grouping(grouping) => self.grouping(grouping),
            Expression::Literal(literal) => self.literal(literal),
            Expression::Unary(unary) => self.unary(unary),
        }
    }

    fn grouping(&self, grouping: &Grouping) -> Result<Value, InterpreterError> {
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

    fn unary(&self, unary: &Unary) -> Result<Value, InterpreterError> {
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

    fn binary(&self, binary: &Binary) -> Result<Value, InterpreterError> {
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
    use crate::tokens::Token;

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

        let interpreter = Interpreter::new(&expression);
        let result = interpreter.evaluate().unwrap();
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

        let interpreter = Interpreter::new(&expression);
        let result = interpreter.evaluate().unwrap();
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

        let interpreter = Interpreter::new(&expression);
        let result = interpreter.evaluate().unwrap();
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

        let interpreter = Interpreter::new(&expression);
        let result = interpreter.evaluate().unwrap();
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

        let interpreter = Interpreter::new(&expression);
        let result = interpreter.evaluate().unwrap();
        assert_eq!(result, Value::Number(17.0));
    }
}
