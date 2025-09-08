use std::io::Write;

use crate::chunk::Chunk;
use crate::chunk::OpCode;

pub struct VirtualMachine {
    chunk: Chunk,
    ip: usize,
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
        }
    }

    pub fn interpret<T: Write + ?Sized>(
        &mut self,
        output: &mut T,
        chunk: Chunk,
    ) -> Result<InterpretResult, String> {
        self.chunk = chunk;
        self.ip = 0;
        self.run(output)
    }

    fn run<T: Write + ?Sized>(&mut self, output: &mut T) -> Result<InterpretResult, String> {
        while self.ip < self.chunk.code.len() {
            // TODO: Only build conditionally
            self.chunk.disassemble_instruction(output, self.ip);
            let instruction = self.read_byte();
            match instruction {
                x if x == OpCode::Return as u8 => {
                    return Ok(InterpretResult::Ok);
                }
                x if x == OpCode::Constant as u8 => {
                    let constant = self.read_constant();
                    writeln!(output, "{}", constant).unwrap();
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
        vm.interpret(&mut output_writer, chunk).unwrap();

        let result = String::from_utf8_lossy(&output.borrow()).to_string();
        assert_eq!(result, "0000 0001 OP_RETURN\n");
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
        vm.interpret(&mut output_writer, chunk).unwrap();

        let result = String::from_utf8_lossy(&output.borrow()).to_string();
        assert_eq!(result, "0000 0001 OP_CONSTANT 0000 1.2\n1.2\n");
    }
}
