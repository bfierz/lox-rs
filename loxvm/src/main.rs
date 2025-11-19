mod chunk;
mod compiler;
mod parser;
mod testing;
mod virtualmachine;

use std::env;
use std::io::Write;
use std::process;

use virtualmachine::VirtualMachine;

// Define exit codes constants
const EXIT_CODE_OK: i32 = 0;
const EXIT_CODE_CMD_LINE_ERROR: i32 = 64;
const EXIT_CODE_DATA_ERROR: i32 = 65;
const EXIT_CODE_SCRIPT_ERROR: i32 = 70;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: loxvm [script]");
        process::exit(EXIT_CODE_CMD_LINE_ERROR);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}

fn run_file(filename: &str) {
    match std::fs::read_to_string(filename) {
        Ok(contents) => {
            let mut output = std::io::stdout();
            let mut vm = VirtualMachine::new();
            if let Err(err) = vm.interpret(&mut output, contents) {
                eprintln!("Runtime error: {}", err);
                process::exit(EXIT_CODE_SCRIPT_ERROR);
            }
        }
        Err(err) => {
            eprintln!("Error reading file {}: {}", filename, err);
            process::exit(74);
        }
    }
}

fn run_prompt() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut input = String::new();

    let mut vm = VirtualMachine::new();
    loop {
        print!("> ");
        stdout.flush().expect("Failed to flush stdout");
        input.clear();
        if stdin.read_line(&mut input).is_err() {
            eprintln!("Error reading input");
            continue;
        }
        if input.trim().is_empty() {
            break;
        }
        let mut output = std::io::stdout();
        if let Err(err) = vm.interpret(&mut output, input.trim().to_string()) {
            eprintln!("Runtime error: {}", err);
        }
    }
}
