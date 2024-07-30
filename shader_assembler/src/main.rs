mod lexer;
mod src_error;
mod assembler;
mod instructions;

use lexer::*;
use assembler::*;

fn main() {
    let test_str = include_str!("../test.shasm");
    let lexer_result = run_lexer(test_str);
    let tokens = match lexer_result {
        Ok(tokens) => {
            println!("LEXER FINISHED - tokens:");
            println!("");
            for token in tokens.iter() {
                if token.t != TokenType::Whitespace {
                    println!("* {:>3}, {:>3}: {:<40} {}", token.line, token.column, format!("{:?}", token.t), if token.value.is_some() { format!("value: {}", token.value.clone().unwrap()) } else { "".to_string() } );
                }
            }
            tokens
        },
        Err(errors) => {
            for error in errors {
                println!("LEXER ERROR - line {:>3}, col {:>3}: {}", error.line, error.column, error.message);
            }
            return;
        }
    };
    let entry_type = EntryType::Vertex;
    let mut start_token = None;
    let mut end_token = None;
    for i in 0..tokens.len() {
        match tokens[i].t {
            TokenType::Command(CommandType::Entry(command_entry_type)) => {
                if start_token.is_none() {
                    if command_entry_type == entry_type {
                        start_token = Some(i);
                    }
                } else {
                    end_token = Some(i);
                }
            },
            _ => {}
        }
    }
    let start = match start_token {
        Some(i) => i,
        None => panic!("No entry point found for {:?}", entry_type),
    };
    let end = end_token.unwrap_or(tokens.len());

    println!("");
    println!("");
    
    let assembler_result = run_assembler(&tokens[start..end], entry_type);
    match assembler_result {
        Ok(bytes) => {
            println!("ASSEMBLY FINISHED - bytecode: ");
            println!("");
            for byte in bytes.iter() {
                println!("{:02X}", &byte);
            }
        },
        Err(errors) => {
            for error in errors {
                println!("ASSEMBER ERROR - line {:>3}, col {:>3}: {}", error.line, error.column, error.message);
            }
        }
    }
}
