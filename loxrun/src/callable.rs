use crate::interpreter::{Interpreter, InterpreterError};
use crate::stmt::FunctionStmt;
use crate::tokens::LiteralTypes;

#[derive(Debug, Clone, PartialEq)]
pub enum Callable {
    LoxFunction(LoxFunction),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoxFunction {
    pub declaration: Box<FunctionStmt>,
}

pub trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<LiteralTypes>,
    ) -> Result<LiteralTypes, InterpreterError>;
}
