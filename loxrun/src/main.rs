use std::env;
use std::process;
use std::fs;
use std::io::{self, Write};

mod expression;
mod interpreter;
mod parser;
mod printer;
mod scanner;
mod stmt;
mod tokens;

use parser::Parser;
use scanner::Scanner;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: rlox [script]");
        process::exit(64);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}

fn run_file(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(contents) => {
            let har_error = run(contents);
            if har_error {
                process::exit(65);
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

        run(input.clone());
    }
}

fn run(source: String) -> bool {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().clone();
    if scanner.had_error {
        return true;
    }

    let mut parser = Parser::new(tokens);
    let parse_result = parser.parse();

    if let Err(err) = parse_result {
        eprintln!("{}", err.message);
        return true;
    }

    let statements = parse_result.unwrap();

    let mut interpreter = interpreter::Interpreter::new(&statements);
    let result = interpreter.execute();
    if let Err(err) = result {
        eprintln!("{}", err.message);
        return true;
    }
    // Print the expression tree
    // This is just for debugging purposes
    // In a real interpreter, you might not want to print the entire tree
    // but rather just the result of the evaluation
    // You can comment this out if you don't want to see the tree
    // let pretty = printer::pretty_print(&expr);
    // println!("{}", pretty);

    false
}
