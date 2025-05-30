use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

mod callable;
mod expression;
mod interpreter;
mod parser;
mod printer;
mod scanner;
mod stmt;
mod tokens;

use parser::Parser;
use scanner::Scanner;

// Define exit codes constants
const EXIT_CODE_OK: i32 = 0;
const EXIT_CODE_CMD_LINE_ERROR: i32 = 64;
const EXIT_CODE_DATA_ERROR: i32 = 65;
const EXIT_CODE_SCRIPT_ERROR: i32 = 70;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: rlox [script]");
        process::exit(EXIT_CODE_CMD_LINE_ERROR);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}

fn run_file(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(contents) => {
            let mut interpreter = interpreter::Interpreter::new();
            let error_code = run(&mut interpreter, contents);
            if error_code != 0 {
                process::exit(error_code);
            }
        }
        Err(err) => {
            eprintln!("Error reading file {}: {}", filename, err);
            process::exit(74);
        }
    }
}

fn run_prompt() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::new();

    let mut interpreter = interpreter::Interpreter::new();
    loop {
        print!("> ");
        stdout.flush().expect("Failed to flush stdout");

        input.clear();
        if stdin.read_line(&mut input).is_err() {
            eprintln!("Error reading input");
            break;
        }

        if input.trim().is_empty() {
            break;
        }

        run(&mut interpreter, input.clone());
    }
}

fn run(interpreter: &mut interpreter::Interpreter, source: String) -> i32 {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().clone();

    let mut parser = Parser::new(tokens);
    let parse_result = parser.parse();

    if scanner.had_error {
        return EXIT_CODE_DATA_ERROR;
    }
    if let Err(_) = parse_result {
        return EXIT_CODE_DATA_ERROR;
    }

    let statements = parse_result.unwrap();

    let result = interpreter.execute(&statements);
    if let Err(err) = result {
        eprintln!("{}", err.message);
        return EXIT_CODE_SCRIPT_ERROR;
    }
    // Print the expression tree
    // This is just for debugging purposes
    // In a real interpreter, you might not want to print the entire tree
    // but rather just the result of the evaluation
    // You can comment this out if you don't want to see the tree
    // let pretty = printer::pretty_print(&expr);
    // println!("{}", pretty);

    EXIT_CODE_OK
}
