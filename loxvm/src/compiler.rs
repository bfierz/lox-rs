use liblox::scanner::Scanner;
use liblox::tokens::TokenType;

use crate::hash_table::HashTable;
use crate::parser::Parser;
use std::cell::RefCell;
use std::rc::Rc;

pub fn compile(
    source: String,
    strings: Rc<RefCell<HashTable>>,
) -> Result<crate::chunk::Chunk, String> {
    let mut scanner = Scanner::new(source);
    let mut parser = Parser::new(scanner.scan_tokens().to_vec(), strings);

    while !parser.match_token(&[TokenType::Eof]) {
        parser.declaration();
    }
    Ok(parser.chunk)
}
