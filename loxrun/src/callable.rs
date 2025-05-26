use crate::interpreter::{Interpreter, InterpreterError, InterpreterResult, Value};
use crate::stmt::FunctionStmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Callable {
    Function(LoxFunction),
}

pub trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, InterpreterError>;
    fn to_string(&self) -> String;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoxFunction {
    pub declaration: Box<FunctionStmt>,
}
impl LoxFunction {
    pub fn new(declaration: FunctionStmt) -> Self {
        Self {
            declaration: Box::new(declaration),
        }
    }
}
impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, InterpreterError> {
        // Create a deep copy of the global environment interpreter.globals
        let fun_env = std::rc::Rc::new(std::cell::RefCell::new(
            interpreter.globals.borrow().deep_clone(),
        ));

        // Add the function's parameters to the new environment
        for (i, arg) in arguments.iter().enumerate() {
            fun_env
                .borrow_mut()
                .define(self.declaration.params[i].lexeme.clone(), arg.clone());
        }
        let result = interpreter.execute_block(&self.declaration.body, fun_env);
        match result {
            Ok(InterpreterResult::Return(value)) => Ok(value),
            Ok(InterpreterResult::None) => Ok(Value::Nil),
            Err(err) => Err(err),
        }
    }

    fn to_string(&self) -> String {
        format!("<fn {}>", self.declaration.name.lexeme)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoxBuiltinFunctionClock {}
impl LoxBuiltinFunctionClock {
    pub fn new() -> Self {
        Self {}
    }
}
impl LoxCallable for LoxBuiltinFunctionClock {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: Vec<Value>,
    ) -> Result<Value, InterpreterError> {
        Ok(Value::Number(lox_clock()))
    }

    fn to_string(&self) -> String {
        "<native fn>".to_string()
    }
}

fn lox_clock() -> f64 {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    duration.as_secs_f64()
}
