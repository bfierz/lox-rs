use crate::expression::Expression;

#[derive(Clone)]
pub enum Stmt {
    Expression(ExpressionStmt),
    Print(PrintStmt),
}

#[derive(Clone)]
pub struct ExpressionStmt {
    pub expression: Box<Expression>,
}

#[derive(Clone)]
pub struct PrintStmt {
    pub expression: Box<Expression>,
}
