use crate::tokens::{LiteralTypes, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Assign(Assign),
    Binary(Binary),
    Call(Call),
    Get(Get),
    Grouping(Grouping),
    Literal(Literal),
    Logical(Logical),
    Set(Set),
    Super(Super),
    This(This),
    Unary(Unary),
    Variable(Variable),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assign {
    pub id: usize,
    pub name: Token,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binary {
    pub id: usize,
    pub left: Box<Expression>,
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub id: usize,
    pub callee: Box<Expression>,
    pub paren: Token,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Get {
    pub id: usize,
    pub object: Box<Expression>,
    pub name: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Grouping {
    pub id: usize,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    pub id: usize,
    pub value: LiteralTypes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Logical {
    pub id: usize,
    pub left: Box<Expression>,
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Set {
    pub id: usize,
    pub object: Box<Expression>,
    pub name: Token,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Super {
    pub id: usize,
    pub keyword: Token,
    pub method: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct This {
    pub id: usize,
    pub keyword: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unary {
    pub id: usize,
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub id: usize,
    pub name: Token,
}
