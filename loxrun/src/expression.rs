use crate::tokens::{LiteralTypes, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Assign(Assign),
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Logical(Logical),
    Unary(Unary),
    Variable(Variable),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assign {
    pub name: Token,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binary {
    pub left: Box<Expression>,
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Grouping {
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    pub value: LiteralTypes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Logical {
    pub left: Box<Expression>,
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: Token,
}
