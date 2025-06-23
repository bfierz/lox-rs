use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::callable::{Callable, LoxCallable, LoxFunction};
use crate::interpreter::{Interpreter, InterpreterError, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    pub name: String,
    pub methods: HashMap<String, Box<LoxFunction>>,
}
impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, Box<LoxFunction>>) -> Self {
        Self { name, methods }
    }

    pub fn find_method(&self, name: &String) -> Option<Box<LoxFunction>> {
        self.methods.get(name).cloned()
    }

    pub fn to_string(&self) -> String {
        format!("{}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    pub class: LoxClass,
    pub fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(class: LoxClass) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &String) -> Result<Value, InterpreterError> {
        if self.fields.contains_key(name) {
            return Ok(self.fields[name].clone());
        }
        if let Some(method) = self.class.find_method(name) {
            return Ok(Value::Callable(Callable::Function(*method)));
        }

        Err(InterpreterError {
            message: format!("Undefined property '{}'.", name),
        })
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.fields.insert(name, value);
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
        let instance = Rc::new(RefCell::new(Instance::new(self.clone())));
        Ok(Value::Instance(instance))
    }

    fn to_string(&self) -> String {
        self.to_string()
    }
}
