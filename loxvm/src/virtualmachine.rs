use std::io::Write;

use crate::chunk::Chunk;
use crate::chunk::OpCode;
use crate::compiler;

pub struct VirtualMachine {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

pub enum Value {
    Number(f64),
    Bool(bool),
    Nil,
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
            stack: Vec::new(),
        }
    }

    pub fn interpret<T: Write + ?Sized>(
        &mut self,
        output: &mut T,
        source: String,
    ) -> Result<InterpretResult, String> {
        self.chunk = compiler::compile(source)?;
        self.ip = 0;
        self.run(output)
    }

    fn run<T: Write + ?Sized>(&mut self, output: &mut T) -> Result<InterpretResult, String> {
        while self.ip < self.chunk.code.len() {
            #[cfg(any(test, feature = "debug_trace"))]
            {
                write!(output, "          ").unwrap();
                for slot in &self.stack {
                    write!(output, "[ ").unwrap();
                    write!(output, "{}", slot).unwrap();
                    write!(output, " ]").unwrap();
                }
                writeln!(output, "").unwrap();
                self.chunk.disassemble_instruction(output, self.ip);
            }
            let instruction = self.read_byte();
            match instruction {
                x if x == OpCode::Return as u8 => {
                    let _ = self.stack.pop().map_or((), |value| {
                        writeln!(output, "{}", value).unwrap();
                    });
                    return Ok(InterpretResult::Ok);
                }
                x if x == OpCode::Nil as u8 => self.stack.push(Value::Nil),
                x if x == OpCode::True as u8 => self.stack.push(Value::Bool(true)),
                x if x == OpCode::False as u8 => self.stack.push(Value::Bool(false)),
                x if x == OpCode::Equal as u8 => self.equal_op(),
                x if x == OpCode::Greater as u8 => self.binary_op(|a, b| Value::Bool(a > b)),
                x if x == OpCode::Less as u8 => self.binary_op(|a, b| Value::Bool(a < b)),
                x if x == OpCode::Add as u8 => self.binary_op(|a, b| Value::Number(a + b)),
                x if x == OpCode::Subtract as u8 => self.binary_op(|a, b| Value::Number(a - b)),
                x if x == OpCode::Multiply as u8 => self.binary_op(|a, b| Value::Number(a * b)),
                x if x == OpCode::Divide as u8 => self.binary_op(|a, b| Value::Number(a / b)),
                x if x == OpCode::Not as u8 => self.not_op(),
                x if x == OpCode::Negate as u8 => self.unary_op(|a| -a),
                x if x == OpCode::Constant as u8 => {
                    let constant = self.read_constant();
                    self.stack.push(Value::Number(constant));
                }
                _ => {
                    return Err(format!("Unknown opcode {}", instruction));
                }
            }
        }
        Ok(InterpretResult::Ok)
    }

    fn read_byte(&mut self) -> u8 {
        let instr = self.chunk.code[self.ip];
        self.ip += 1;
        instr
    }

    fn read_constant(&mut self) -> f64 {
        let constant_index = self.read_byte() as usize;
        self.chunk.constants[constant_index]
    }

    fn binary_op(&mut self, op: fn(f64, f64) -> Value) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        let Value::Number(b) = b else {
            panic!("Operand must be a number.");
        };
        let Value::Number(a) = a else {
            panic!("Operand must be a number.");
        };
        self.stack.push(op(a, b));
    }

    fn binary_logic_op(&mut self, op: fn(bool, bool) -> bool) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        let Value::Bool(b) = b else {
            panic!("Operand must be a boolean.");
        };
        let Value::Bool(a) = a else {
            panic!("Operand must be a boolean.");
        };
        self.stack.push(Value::Bool(op(a, b)));
    }

    fn unary_op(&mut self, op: fn(f64) -> f64) {
        let a = self.stack.pop().unwrap();
        let Value::Number(a) = a else {
            panic!("Operand must be a number.");
        };
        self.stack.push(Value::Number(op(a)));
    }

    fn not_op(&mut self) {
        let a = self.stack.pop().unwrap();
        self.stack.push(Value::Bool(Self::is_falsey(&a)));
    }

    fn equal_op(&mut self) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(Value::Bool(self.valuesEqual(&a, &b)));
    }

    fn valuesEqual(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            _ => false,
        }
    }

    fn is_falsey(value: &Value) -> bool {
        match value {
            Value::Nil => true,
            Value::Bool(b) => !*b,
            _ => false,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
        }
    }
}

mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::testing::*;

    #[test]
    fn test_return_op() {
        let output = Rc::new(RefCell::new(Vec::<u8>::new()));
        let mut output_writer = Box::new(VecWriter(Rc::clone(&output)));

        let mut chunk = Chunk::new();
        chunk.write_op_code(OpCode::Return, 1);

        let mut vm = VirtualMachine::new();
        vm.chunk = chunk;
        vm.run(&mut output_writer).unwrap();

        let result = String::from_utf8_lossy(&output.borrow()).to_string();
        assert_eq!(result, "          \n0000 0001 OP_RETURN\n");
    }
    #[test]
    fn test_constant_op() {
        let output = Rc::new(RefCell::new(Vec::<u8>::new()));
        let mut output_writer = Box::new(VecWriter(Rc::clone(&output)));

        let mut chunk = Chunk::new();
        let constant_index = chunk.add_constant(1.2);
        chunk.write_op_code(OpCode::Constant, 1);
        chunk.write(constant_index as u8, 1);

        let mut vm = VirtualMachine::new();
        vm.chunk = chunk;
        vm.run(&mut output_writer).unwrap();

        let result = String::from_utf8_lossy(&output.borrow()).to_string();
        assert_eq!(result, "          \n0000 0001 OP_CONSTANT 0000 1.2\n");
    }
    #[test]
    fn test_negate_op() {
        let output = Rc::new(RefCell::new(Vec::<u8>::new()));
        let mut output_writer = Box::new(VecWriter(Rc::clone(&output)));

        let mut chunk = Chunk::new();
        let constant_index = chunk.add_constant(1.2);
        chunk.write_op_code(OpCode::Constant, 1);
        chunk.write(constant_index as u8, 1);
        chunk.write_op_code(OpCode::Negate, 2);
        chunk.write_op_code(OpCode::Return, 3);

        let mut vm = VirtualMachine::new();
        vm.chunk = chunk;
        vm.run(&mut output_writer).unwrap();

        let result = String::from_utf8_lossy(&output.borrow()).to_string();
        assert_eq!(
            result,
            "          \n0000 0001 OP_CONSTANT 0000 1.2\
            \n          [ 1.2 ]\n0002 0002 OP_NEGATE\
            \n          [ -1.2 ]\n0003 0003 OP_RETURN\n-1.2\n"
        );
    }
    #[test]
    fn test_add_op() {
        let output = Rc::new(RefCell::new(Vec::<u8>::new()));
        let mut output_writer = Box::new(VecWriter(Rc::clone(&output)));

        let mut chunk = Chunk::new();
        let constant_index = chunk.add_constant(1.2);
        chunk.write_op_code(OpCode::Constant, 1);
        chunk.write(constant_index as u8, 1);
        chunk.write_op_code(OpCode::Constant, 1);
        chunk.write(constant_index as u8, 1);
        chunk.write_op_code(OpCode::Add, 1);
        chunk.write_op_code(OpCode::Return, 2);

        let mut vm = VirtualMachine::new();
        vm.chunk = chunk;
        vm.run(&mut output_writer).unwrap();

        let result = String::from_utf8_lossy(&output.borrow()).to_string();
        assert_eq!(
            result,
            "          \
            \n0000 0001 OP_CONSTANT 0000 1.2\
            \n          [ 1.2 ]\
            \n0002    | OP_CONSTANT 0000 1.2\
            \n          [ 1.2 ][ 1.2 ]\
            \n0004    | OP_ADD\
            \n          [ 2.4 ]\
            \n0005 0002 OP_RETURN\
            \n2.4\n"
        );
    }

    #[test]
    fn test_arithmatic() {
        let output = Rc::new(RefCell::new(Vec::<u8>::new()));
        let mut output_writer = Box::new(VecWriter(Rc::clone(&output)));

        let mut chunk = Chunk::new();

        //  -( (1.2 + 3.4) / 5.6 )
        let constant_a = chunk.add_constant(1.2);
        chunk.write_op_code(OpCode::Constant, 123);
        chunk.write(constant_a as u8, 123);

        let constant_b = chunk.add_constant(3.4);
        chunk.write_op_code(OpCode::Constant, 123);
        chunk.write(constant_b as u8, 123);

        chunk.write_op_code(OpCode::Add, 123);

        let constant_c = chunk.add_constant(5.6);
        chunk.write_op_code(OpCode::Constant, 123);
        chunk.write(constant_c as u8, 123);

        chunk.write_op_code(OpCode::Divide, 123);
        chunk.write_op_code(OpCode::Negate, 123);
        chunk.write_op_code(OpCode::Return, 123);

        let mut vm = VirtualMachine::new();
        vm.chunk = chunk;
        vm.run(&mut output_writer).unwrap();

        let result = String::from_utf8_lossy(&output.borrow()).to_string();
        assert_eq!(
            result,
            "          \
            \n0000 0123 OP_CONSTANT 0000 1.2\
            \n          [ 1.2 ]\
            \n0002    | OP_CONSTANT 0001 3.4\
            \n          [ 1.2 ][ 3.4 ]\
            \n0004    | OP_ADD\
            \n          [ 4.6 ]\
            \n0005    | OP_CONSTANT 0002 5.6\
            \n          [ 4.6 ][ 5.6 ]\
            \n0007    | OP_DIVIDE\
            \n          [ 0.8214285714285714 ]\
            \n0008    | OP_NEGATE\
            \n          [ -0.8214285714285714 ]\
            \n0009    | OP_RETURN\
            \n-0.8214285714285714\n"
        );
    }
}
