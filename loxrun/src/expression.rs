use crate::tokens::{LiteralTypes, Token};

#[derive(Debug, Clone)]
pub enum Expression {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub left: Box<Expression>,
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct Grouping {
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub value: LiteralTypes,
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expression>,
}
