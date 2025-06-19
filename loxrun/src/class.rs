use crate::callable::LoxCallable;
use crate::interpreter::{Interpreter, InterpreterError, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    pub name: String,
}
impl LoxClass {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn to_string(&self) -> String {
        format!("{}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    pub class: LoxClass,
}

impl Instance {
    pub fn new(class: LoxClass) -> Self {
        Self { class }
    }

    pub fn to_string(&self) -> String {
        format!("{} instance", self.class.to_string())
    }
}

impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        0 // Class constructors don't take any arguments
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: Vec<Value>,
    ) -> Result<Value, InterpreterError> {
        let instance = Instance::new(self.clone());
        Ok(Value::Instance(instance))
    }

    fn to_string(&self) -> String {
        self.to_string()
    }
}
