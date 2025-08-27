use std::cell::RefCell;
use std::io;
use std::io::Write;
use std::rc::Rc;

// Mocking the output stream for testing
pub struct VecWriter(pub Rc<RefCell<Vec<u8>>>);

impl Write for VecWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
