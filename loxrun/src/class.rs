use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use liblox::tokens::Token;

use crate::callable::{Callable, LoxCallable, LoxFunction};
use crate::interpreter::{Interpreter, InterpreterError, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    pub name: String,
    pub superclass: Option<Rc<RefCell<LoxClass>>>,
    pub methods: HashMap<String, Box<LoxFunction>>,
}
impl LoxClass {
    pub fn new(
        name: String,
        superclass: Option<Rc<RefCell<LoxClass>>>,
        methods: HashMap<String, Box<LoxFunction>>,
    ) -> Self {
        Self {
            name,
            superclass,
            methods,
        }
    }

    pub fn find_method(&self, name: &String) -> Option<Box<LoxFunction>> {
        self.methods.get(name).cloned().or_else(|| {
            self.superclass
                .as_ref()
                .and_then(|superclass| superclass.borrow().find_method(name))
        })
    }

    pub fn to_string(&self) -> String {
        format!("{}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    pub class: Rc<RefCell<LoxClass>>,
    pub fields: HashMap<String, Value>,
}

pub fn get_instance_field(
    instance: &Rc<RefCell<Instance>>,
    name: &Token,
) -> Result<Value, InterpreterError> {
    if instance.borrow().fields.contains_key(&name.lexeme) {
        return Ok(instance.borrow().fields[&name.lexeme].clone());
    }
    if let Some(method) = instance.borrow().class.borrow().find_method(&name.lexeme) {
        return Ok(Value::Callable(Callable::Function(method.bind(&instance))));
    }

    Err(InterpreterError {
        message: format!(
            "Undefined property '{}'.\n[line {}]",
            name.lexeme, name.line
        ),
    })
}

impl Instance {
    pub fn new(class: Rc<RefCell<LoxClass>>) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.fields.insert(name, value);
    }

    pub fn to_string(&self) -> String {
        format!("{} instance", self.class.borrow().to_string())
    }
}

impl LoxCallable for Rc<RefCell<LoxClass>> {
    fn arity(&self) -> usize {
        self.borrow()
            .find_method(&"init".to_string())
            .map_or(0, |method| method.arity())
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, InterpreterError> {
        let instance = Rc::new(RefCell::new(Instance::new(self.clone())));
        let method = self.borrow().find_method(&"init".to_string());
        if let Some(method) = method {
            let method = method.bind(&instance);
            method.call(interpreter, arguments)?;
        }
        Ok(Value::Instance(instance))
    }

    fn to_string(&self) -> String {
        self.borrow().to_string()
    }
}
