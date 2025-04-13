use crate::{expression::Expression, tokens::Token};

#[derive(Clone)]
pub enum Stmt {
    Expression(ExpressionStmt),
    If(IfStmt),
    Print(PrintStmt),
    Block(BlockStmt),
    Var(VarStmt),
}

#[derive(Clone)]
pub struct ExpressionStmt {
    pub expression: Box<Expression>,
}

#[derive(Clone)]
pub struct IfStmt {
    pub condition: Box<Expression>,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Clone)]
pub struct PrintStmt {
    pub expression: Box<Expression>,
}

#[derive(Clone)]
pub struct BlockStmt {
    pub statements: Vec<Stmt>,
}

#[derive(Clone)]
pub struct VarStmt {
    pub name: Token,
    pub initializer: Option<Box<Expression>>,
}
