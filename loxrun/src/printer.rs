use crate::expression::*;
use crate::tokens::LiteralTypes;

pub fn pretty_print(expr: &Expression) -> String {
    match expr {
        Expression::Binary(binary) => {
            let left = pretty_print(&*binary.left);
            let right = pretty_print(&*binary.right);
            format!("({} {} {})", binary.operator.lexeme, left, right)
        }
        Expression::Grouping(grouping) => {
            let expr = pretty_print(&*grouping.expression);
            format!("(group {})", expr)
        }
        Expression::Literal(literal) => {
            match &literal.value {
                LiteralTypes::String(s) => format!("{}", s),
                LiteralTypes::Number(n) => format!("{}", n),
                LiteralTypes::Bool(b) => format!("{}", b),
                LiteralTypes::Nil => format!("nil"),
            }
        }
        Expression::Unary(unary) => {
            let right = pretty_print(&*unary.right);
            format!("({} {})", unary.operator.lexeme, right)
        }
    }
}

pub fn rpn_print(expr: &Expression) -> String {
    match expr {
        Expression::Binary(binary) => {
            let left = rpn_print(&*binary.left);
            let right = rpn_print(&*binary.right);
            format!("{} {} {}", left, right, binary.operator.lexeme)
        }
        Expression::Grouping(grouping) => {
            rpn_print(&*grouping.expression)
        }
        Expression::Literal(literal) => {
            match &literal.value {
                LiteralTypes::String(s) => format!("{}", s),
                LiteralTypes::Number(n) => format!("{}", n),
                LiteralTypes::Bool(b) => format!("{}", b),
                LiteralTypes::Nil => format!("nil"),
            }
        }
        Expression::Unary(unary) => {
            let right = rpn_print(&*unary.right);
            format!("{} {}", right, unary.operator.lexeme)
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
            left: Box::new(Expression::Unary(Unary {
                operator: Token::new(TokenType::Minus, "-".to_string(), LiteralTypes::Nil, 1),
                right: Box::new(Expression::Literal(Literal {
                    value: LiteralTypes::Number(123.0),
                })),
            })),
            operator: Token::new(TokenType::Star, "*".to_string(), LiteralTypes::Nil, 1),
            right: Box::new(Expression::Grouping(Grouping {
                expression: Box::new(Expression::Literal(Literal {
                    value: LiteralTypes::Number(45.67),
                }))
            })),
        });

        assert_eq!(pretty_print(&expr), "(* (- 123) (group 45.67))");
    }

    #[test]
    fn test_rpn_print() {
        let expr = Expression::Binary(Binary {
            left: Box::new(Expression::Binary(Binary {
                left: Box::new(Expression::Literal(Literal {
                    value: LiteralTypes::Number(1.0),
                })),
                operator: Token::new(TokenType::Plus, "+".to_string(), LiteralTypes::Nil, 1),
                right: Box::new(Expression::Literal(Literal {
                    value: LiteralTypes::Number(2.0),
                })),
            })),
            operator: Token::new(TokenType::Star, "*".to_string(), LiteralTypes::Nil, 1),
            right: Box::new(Expression::Binary(Binary {
                left: Box::new(Expression::Literal(Literal {
                    value: LiteralTypes::Number(4.0),
                })),
                operator: Token::new(TokenType::Minus, "-".to_string(), LiteralTypes::Nil, 1),
                right: Box::new(Expression::Literal(Literal {
                    value: LiteralTypes::Number(3.0),
                })),
            })),
        });

        assert_eq!(rpn_print(&expr), "1 2 + 4 3 - *");
    }
}
