use crate::{expression::Expression, tokens::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(ExpressionStmt),
    Function(FunctionStmt),
    If(IfStmt),
    Print(PrintStmt),
    Block(BlockStmt),
    Var(VarStmt),
    While(WhileStmt),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionStmt {
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionStmt {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub condition: Box<Expression>,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrintStmt {
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStmt {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarStmt {
    pub name: Token,
    pub initializer: Option<Box<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub condition: Box<Expression>,
    pub body: Box<Stmt>,
}
