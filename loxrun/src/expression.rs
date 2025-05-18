use crate::tokens::{LiteralTypes, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Assign(Assign),
    Binary(Binary),
    Call(Call),
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
pub struct Call {
    pub callee: Box<Expression>,
    pub paren: Token,
    pub arguments: Vec<Expression>,
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
