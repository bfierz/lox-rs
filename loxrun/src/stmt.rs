use crate::{expression::Expression, tokens::Token};

#[derive(Clone)]
pub enum Stmt {
    Expression(ExpressionStmt),
    Print(PrintStmt),
    Var(VarStmt),
}

#[derive(Clone)]
pub struct ExpressionStmt {
    pub expression: Box<Expression>,
}

#[derive(Clone)]
pub struct PrintStmt {
    pub expression: Box<Expression>,
}

#[derive(Clone)]
pub struct VarStmt {
    pub name: Token,
    pub initializer: Option<Box<Expression>>,
}
