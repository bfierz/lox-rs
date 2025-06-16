use crate::expression::*;
use crate::tokens::LiteralTypes;

pub fn pretty_print(expr: &Expression) -> String {
    match expr {
        Expression::Assign(assign) => {
            let value = pretty_print(&*assign.value);
            format!("{} = {}", assign.name.lexeme, value)
        }
        Expression::Binary(binary) => {
            let left = pretty_print(&*binary.left);
            let right = pretty_print(&*binary.right);
            format!("({} {} {})", binary.operator.lexeme, left, right)
        }
        Expression::Call(call) => {
            let callee = pretty_print(&*call.callee);
            let args: Vec<String> = call.arguments.iter().map(|arg| pretty_print(arg)).collect();
            format!("{}({})", callee, args.join(", "))
        }
        Expression::Grouping(grouping) => {
            let expr = pretty_print(&*grouping.expression);
            format!("(group {})", expr)
        }
        Expression::Literal(literal) => match &literal.value {
            LiteralTypes::String(s) => format!("{}", s),
            LiteralTypes::Number(n) => format!("{}", n),
            LiteralTypes::Bool(b) => format!("{}", b),
            LiteralTypes::Nil => format!("nil"),
        },
        Expression::Logical(logical) => {
            let left = pretty_print(&*logical.left);
            let right = pretty_print(&*logical.right);
            format!("({} {} {})", logical.operator.lexeme, left, right)
        }
        Expression::Unary(unary) => {
            let right = pretty_print(&*unary.right);
            format!("({} {})", unary.operator.lexeme, right)
        }
        Expression::Variable(variable) => {
            format!("{}", variable.name)
        }
    }
}

pub fn rpn_print(expr: &Expression) -> String {
    match expr {
        Expression::Assign(assign) => {
            let value = rpn_print(&*assign.value);
            format!("{} = {}", assign.name.lexeme, value)
        }
        Expression::Binary(binary) => {
            let left = rpn_print(&*binary.left);
            let right = rpn_print(&*binary.right);
            format!("{} {} {}", left, right, binary.operator.lexeme)
        }
        Expression::Call(call) => {
            let callee = rpn_print(&*call.callee);
            let args: Vec<String> = call.arguments.iter().map(|arg| rpn_print(arg)).collect();
            format!("{}({})", callee, args.join(", "))
        }
        Expression::Grouping(grouping) => rpn_print(&*grouping.expression),
        Expression::Literal(literal) => match &literal.value {
            LiteralTypes::String(s) => format!("{}", s),
            LiteralTypes::Number(n) => format!("{}", n),
            LiteralTypes::Bool(b) => format!("{}", b),
            LiteralTypes::Nil => format!("nil"),
        },
        Expression::Logical(logical) => {
            let left = rpn_print(&*logical.left);
            let right = rpn_print(&*logical.right);
            format!("{} {} {}", left, right, logical.operator.lexeme)
        }
        Expression::Unary(unary) => {
            let right = rpn_print(&*unary.right);
            format!("{} {}", right, unary.operator.lexeme)
        }
        Expression::Variable(variable) => {
            format!("{}", variable.name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::{Token, TokenType};

    #[test]
    fn test_pretty_print() {
        let expr = Expression::Binary(Binary {
            id: 0,
            left: Box::new(Expression::Unary(Unary {
                id: 1,
                operator: Token::new(TokenType::Minus, "-".to_string(), LiteralTypes::Nil, 1),
                right: Box::new(Expression::Literal(Literal {
                    id: 2,
                    value: LiteralTypes::Number(123.0),
                })),
            })),
            operator: Token::new(TokenType::Star, "*".to_string(), LiteralTypes::Nil, 1),
            right: Box::new(Expression::Grouping(Grouping {
                id: 3,
                expression: Box::new(Expression::Literal(Literal {
                    id: 4,
                    value: LiteralTypes::Number(45.67),
                })),
            })),
        });

        assert_eq!(pretty_print(&expr), "(* (- 123) (group 45.67))");
    }

    #[test]
    fn test_rpn_print() {
        let expr = Expression::Binary(Binary {
            id: 0,
            left: Box::new(Expression::Binary(Binary {
                id: 1,
                left: Box::new(Expression::Literal(Literal {
                    id: 2,
                    value: LiteralTypes::Number(1.0),
                })),
                operator: Token::new(TokenType::Plus, "+".to_string(), LiteralTypes::Nil, 1),
                right: Box::new(Expression::Literal(Literal {
                    id: 3,
                    value: LiteralTypes::Number(2.0),
                })),
            })),
            operator: Token::new(TokenType::Star, "*".to_string(), LiteralTypes::Nil, 1),
            right: Box::new(Expression::Binary(Binary {
                id: 4,
                left: Box::new(Expression::Literal(Literal {
                    id: 5,
                    value: LiteralTypes::Number(4.0),
                })),
                operator: Token::new(TokenType::Minus, "-".to_string(), LiteralTypes::Nil, 1),
                right: Box::new(Expression::Literal(Literal {
                    id: 6,
                    value: LiteralTypes::Number(3.0),
                })),
            })),
        });

        assert_eq!(rpn_print(&expr), "1 2 + 4 3 - *");
    }
}
