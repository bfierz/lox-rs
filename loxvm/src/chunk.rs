use std::io::Write;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Constant = 0,
    Add = 1,
    Subtract = 2,
    Multiply = 3,
    Divide = 4,
    Negate = 5,
    Return = 6,
}

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<f64>,
    pub lines: Vec<u32>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn write_op_code(&mut self, op_code: OpCode, line: u32) {
        self.code.push(op_code as u8);
        self.lines.push(line);
    }

    pub fn write(&mut self, byte: u8, line: u32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: f64) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn disassemble<T: Write + ?Sized>(&self, output: &mut T, name: &str) {
        writeln!(output, "== {} ==", name).unwrap();

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(output, offset);
        }
    }

    pub fn disassemble_instruction<T: Write + ?Sized>(
        &self,
        output: &mut T,
        offset: usize,
    ) -> usize {
        write!(output, "{:04} ", offset).unwrap();

        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            write!(output, "   | ").unwrap();
        } else {
            write!(output, "{:04} ", self.lines[offset]).unwrap();
        }

        let instruction = unsafe { ::std::mem::transmute(self.code[offset]) };
        match instruction {
            OpCode::Constant => self.disassemble_constant_instruction(output, offset),
            OpCode::Add => self.disassemble_simple_instruction(output, "OP_ADD", offset),
            OpCode::Subtract => self.disassemble_simple_instruction(output, "OP_SUBTRACT", offset),
            OpCode::Multiply => self.disassemble_simple_instruction(output, "OP_MULTIPLY", offset),
            OpCode::Divide => self.disassemble_simple_instruction(output, "OP_DIVIDE", offset),
            OpCode::Negate => self.disassemble_simple_instruction(output, "OP_NEGATE", offset),
            OpCode::Return => self.disassemble_simple_instruction(output, "OP_RETURN", offset),
            //_ => {
            //    writeln!(output, "Unknown opcode {}", instruction as u8).unwrap();
            //    offset + 1
            //}
        }
    }

    fn disassemble_constant_instruction<T: Write + ?Sized>(
        &self,
        output: &mut T,
        offset: usize,
    ) -> usize {
        let constant_index = self.code[offset + 1] as usize;
        let constant_value = self.constants[constant_index];
        writeln!(
            output,
            "OP_CONSTANT {:04} {}",
            constant_index, constant_value
        )
        .unwrap();
        offset + 2
    }

    fn disassemble_simple_instruction<T: Write + ?Sized>(
        &self,
        output: &mut T,
        name: &str,
        offset: usize,
    ) -> usize {
        writeln!(output, "{}", name).unwrap();
        offset + 1
    }
}

mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::testing::*;

    #[test]
    fn test_disassemble_chunk_return() {
        let output = Rc::new(RefCell::new(Vec::<u8>::new()));
        let mut output_writer = Box::new(VecWriter(Rc::clone(&output)));

        let mut chunk = Chunk::new();
        chunk.write_op_code(OpCode::Return, 1);
        chunk.disassemble(&mut output_writer, "test chunk");

        let result = String::from_utf8_lossy(&output.borrow()).to_string();
        assert_eq!(result, "== test chunk ==\n0000 0001 OP_RETURN\n");
    }
    #[test]
    fn test_disassemble_chunk_constant() {
        let output = Rc::new(RefCell::new(Vec::<u8>::new()));
        let mut output_writer = Box::new(VecWriter(Rc::clone(&output)));

        let mut chunk = Chunk::new();
        let constant_index = chunk.add_constant(1.2);
        chunk.write_op_code(OpCode::Constant, 1);
        chunk.write(constant_index as u8, 1);
        chunk.disassemble(&mut output_writer, "test chunk");

        let result = String::from_utf8_lossy(&output.borrow()).to_string();
        assert_eq!(result, "== test chunk ==\n0000 0001 OP_CONSTANT 0000 1.2\n");
    }
}
