mod lexer;
mod src_error;
mod assembler;
mod instructions;

use lexer::*;
use assembler::*;

const PROGRAM_USAGE: &'static str = "usage:\n    shasm <shader assmebly file> -<shader kind> <binary output file>\n        shader kinds:\n        * v: vertex\n        * f: fragment\n        * c: compute\n";

fn main() {
    let mut args = std::env::args();
    args.next();
    let assembly_file = args.next().expect(PROGRAM_USAGE);
    let mode_switch = args.next().expect(PROGRAM_USAGE);
    let output_binary_file = args.next().expect(PROGRAM_USAGE);

    let entry_type = match mode_switch.as_str() {
        "-v" => AssemblyMode::Vertex,
        "-f" => AssemblyMode::Fragment,
        "-c" => AssemblyMode::Compute,
        _ => panic!("Unknown flag: {}", mode_switch),
    };

    let input = std::fs::read_to_string(assembly_file).expect("Failed to read input file!");

    let lexer_result = run_lexer(&input);
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
    
    let assembler_result = run_assembler(&tokens[..], entry_type);
    match assembler_result {
        Ok(bytes) => {
            println!("ASSEMBLY FINISHED - bytecode: ");
            println!("");
            for byte in bytes.iter() {
                println!("{:02X}", &byte);
            }
            std::fs::write(output_binary_file, &bytes).expect("Failed to write output file")
        },
        Err(errors) => {
            for error in errors {
                println!("ASSEMBER ERROR - line {:>3}, col {:>3}: {}", error.line, error.column, error.message);
            }
        }
    }
}
