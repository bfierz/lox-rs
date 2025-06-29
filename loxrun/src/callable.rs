use crate::class::{Instance, LoxClass};
use crate::interpreter::{Environment, Interpreter, InterpreterError, InterpreterResult, Value};
use crate::stmt::FunctionStmt;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Callable {
    DynamicFunction(LoxDynamicFunction),
    Function(LoxFunction),
    Class(LoxClass),
}
impl std::fmt::Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::DynamicFunction(fun) => {
                write!(f, "{}", fun.callable.borrow().as_ref().to_string())
            }
            Callable::Function(fun) => write!(f, "{}", fun.to_string()),
            Callable::Class(class) => write!(f, "{}", class.to_string()),
        }
    }
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

pub struct LoxDynamicFunction {
    pub callable: Rc<RefCell<Box<dyn LoxCallable>>>,
}
impl Clone for LoxDynamicFunction {
    fn clone(&self) -> Self {
        Self {
            callable: Rc::clone(&self.callable),
        }
    }
}
impl fmt::Debug for LoxDynamicFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LoxDynamicFunction {{ callable: {:?} }}",
            self.callable.borrow().to_string()
        )
    }
}
impl PartialEq for LoxDynamicFunction {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.callable, &other.callable)
    }
}

#[derive(Debug, Clone)]
pub struct LoxFunction {
    pub declaration: Box<FunctionStmt>,

    /// The closure is an optional environment that captures the variables from the scope where the function was defined.
    pub closure: Rc<RefCell<Environment>>,

    is_initializer: bool,
}
impl LoxFunction {
    pub fn new(
        declaration: FunctionStmt,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            declaration: Box::new(declaration),
            closure,
            is_initializer,
        }
    }

    pub fn bind(&self, instance: &Rc<RefCell<Instance>>) -> Self {
        let fun_env = Rc::new(RefCell::new(Environment::with_enclosing(
            self.closure.clone(),
        )));
        fun_env
            .borrow_mut()
            .define("this".to_string(), Value::Instance(Rc::clone(instance)));
        Self {
            declaration: self.declaration.clone(),
            closure: fun_env,
            is_initializer: self.is_initializer,
        }
    }
}

impl PartialEq for LoxFunction {
    fn eq(&self, other: &Self) -> bool {
        self.declaration == other.declaration && Rc::ptr_eq(&self.closure, &other.closure)
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
        let fun_env = Rc::new(RefCell::new(Environment::with_enclosing(
            self.closure.clone(),
        )));

        // Add the function's parameters to the new environment
        for (i, arg) in arguments.iter().enumerate() {
            fun_env
                .borrow_mut()
                .define(self.declaration.params[i].lexeme.clone(), arg.clone());
        }
        let result = interpreter.execute_block(&self.declaration.body, fun_env);
        match result {
            Ok(InterpreterResult::None) | Ok(InterpreterResult::Return(Value::Nil)) => {
                if self.is_initializer {
                    // If this function is an initializer, return the instance it was called on
                    let instance = self.closure.borrow().get_at(&"this".to_string(), 0);
                    if let Some(Value::Instance(instance)) = instance {
                        return Ok(Value::Instance(Rc::clone(&instance)));
                    } else {
                        return Err(InterpreterError {
                            message: "Initializer function called without 'this' instance."
                                .to_string(),
                        });
                    }
                }
                Ok(Value::Nil)
            }
            Ok(InterpreterResult::Return(value)) => Ok(value),
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
