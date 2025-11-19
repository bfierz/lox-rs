use liblox::scanner::Scanner;
use liblox::tokens::TokenType;

use crate::parser::Parser;

pub fn compile(source: String) -> Result<crate::chunk::Chunk, String> {
    let mut scanner = Scanner::new(source);
    let mut parser = Parser::new(scanner.scan_tokens().to_vec());
    parser.expression();
    parser.emit_return();
    Ok(parser.chunk)
}
