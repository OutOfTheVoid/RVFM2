use std::collections::HashMap;
use std::hash::Hash;
use std::iter::Peekable;

use super::lexer::*;
use super::src_error::*;
use super::instructions::*;

pub enum AssemblerMode {
    Vertex,
    Fragment,
    Compute,
}

fn expect_token<'t, I: Iterator<Item = &'t Token>>(t: TokenType, iter: &mut Peekable<I>) -> Result<&'t Token, SourceError> {
    if let Some(token) = iter.peek() {
        if token.t == t {
            Ok(iter.next().unwrap())
        } else {
            Err(SourceError {
                message: format!("Expected token with type: {:?}, but found {:?}", t, token.t),
                line: token.line,
                column: token.column,
            })
        }
    } else {
        Err(SourceError {
            message: format!("Expected token with type: {:?}, but found end of file", t),
            line: 0,
            column: 0
        })
    }
}

pub fn try_expect_token<'t, I: Iterator<Item = &'t Token>>(t: TokenType, iter: &mut Peekable<I>) -> Option<&'t Token> {
    if let Some(token) = iter.peek() {
        if token.t == t {
            Some(iter.next().unwrap())
        } else {
            None
        }
    } else {
       None
    }
}

fn expect_token_with<'t, I: Iterator<Item = &'t Token>>(description: &str, test: impl Fn(&TokenType) -> bool, iter: &mut Peekable<I>) -> Result<&'t Token, SourceError> {
    if let Some(token) = iter.peek() {
        if test(&token.t) {
            Ok(iter.next().unwrap())
        } else {
            Err(SourceError {
                message: format!("Expected {}, but found {:?}", description, token.t),
                line: token.line,
                column: token.column,
            })
        }
    } else {
        Err(SourceError {
            message: format!("Expected {}, but found end of file", description),
            line: 0,
            column: 0
        })
    }
}

fn try_expect_token_with<'t, I: Iterator<Item = &'t Token>>(description: &str, test: impl Fn(&TokenType) -> bool, iter: &mut Peekable<I>) -> Option<&'t Token> {
    if let Some(token) = iter.peek() {
        if test(&token.t) {
            Some(iter.next().unwrap())
        } else {
            None
        }
    } else {
        None
    }
}

fn expect_write_register<'t, I: Iterator<Item = &'t Token>>(iter: &mut Peekable<I>, aliases: Option<&HashMap<(AssemblyMode, String), RegisterName>>, assembly_mode: AssemblyMode) -> Result<(RegisterName, &'t Token), SourceError> {
    let token = expect_token_with(&format!("register writable from shader with {:?}", assembly_mode),
        |t| match t {
                TokenType::Register(RegisterName::LocalS(_)) |
                TokenType::Register(RegisterName::LocalV(_)) |
                TokenType::Register(RegisterName::OutputS(_)) |
                TokenType::Register(RegisterName::OutputV(_)) => true,
                TokenType::Register(RegisterName::BuiltinS(scalar_builtin)) => {
                    match (assembly_mode, scalar_builtin) {
                        (AssemblyMode::Fragment, ScalarBuiltin::Depth) => true,
                        _ => false,
                    }
                },
                TokenType::Register(RegisterName::BuiltinV(vector_builtin)) => {
                    match (assembly_mode, vector_builtin) {
                        (AssemblyMode::Vertex, VectorBuiltin::VertexPosition) => true,
                        _ => false,
                    }
                },
                TokenType::Name => aliases.is_some(),
                _ => false,
            },
        iter)?;
    if let Token { t: TokenType::Register(name), .. } = token {
        Ok((*name, token))
    } else if let Token { t: TokenType::Name, value: Some(variable_name), line, column } = token {
        let variables = aliases.unwrap();
        if let Some(register) = variables.get(&(assembly_mode, variable_name.clone())) {
            Ok((*register, token))
        } else {
            Err(SourceError {
                message: format!("No variable with the name {}", variable_name),
                line: *line,
                column: *column
            })
        }
    } else {
        unreachable!()
    }
}

fn expect_read_register<'t, I: Iterator<Item = &'t Token>>(iter: &mut Peekable<I>, aliases: Option<&HashMap<(AssemblyMode, String), RegisterName>>, assembly_mode: AssemblyMode) -> Result<(RegisterName, &'t Token), SourceError> {
    let token = expect_token_with(&format!("register readable from {:?} shader", assembly_mode),
        |t| match t {
                TokenType::Register(RegisterName::ConstantS(_)) |
                TokenType::Register(RegisterName::ConstantV(_)) |
                TokenType::Register(RegisterName::LocalS(_)) |
                TokenType::Register(RegisterName::LocalV(_)) |
                TokenType::Register(RegisterName::InputS(_)) |
                TokenType::Register(RegisterName::InputV(_)) => true,
                TokenType::Register(RegisterName::BuiltinS(scalar_builtin)) => {
                    match (assembly_mode, scalar_builtin) {
                        (AssemblyMode::Vertex, ScalarBuiltin::VertexId)        |
                        (AssemblyMode::Vertex, ScalarBuiltin::ProvokingVertex) |
                        (AssemblyMode::Fragment, ScalarBuiltin::Depth)         => true,
                        _ => false,
                    }
                },
                TokenType::Register(RegisterName::BuiltinV(vector_builtin)) => {
                    match (assembly_mode, vector_builtin) {
                        (AssemblyMode::Fragment, VectorBuiltin::VertexPosition) |
                        (AssemblyMode::Fragment, VectorBuiltin::Barycentric   ) |
                        (AssemblyMode::Fragment, VectorBuiltin::Linear         ) |
                        (AssemblyMode::Fragment, VectorBuiltin::VertexIds     ) => true,
                        _ => false
                    }
                },
                TokenType::Name => aliases.is_some(),
                _ => false,
            },
        iter)?;
    if let Token { t: TokenType::Register(name), .. } = token {
        Ok((*name, token))
    } else if let Token { t: TokenType::Name, value: Some(name), line, column } = token {
        
        if let Some(aliases) = aliases {
            if let Some(register) = aliases.get(&(assembly_mode, name.clone())) {
                Ok((*register, token))
            } else {
                Err(SourceError {
                    message: format!("No variable or constant with the name {}", name),
                    line: *line,
                    column: *column
                })
            }
        } else {
            Err(SourceError {
                message: format!("\"{}\" is not a valid register name", name),
                line: *line,
                column: *column,
            })
        }
    } else {
        unreachable!()
    }
}



pub fn run_assembler(tokens: &[Token], entry_mode: AssemblyMode) -> Result<Box<[u8]>, Vec<SourceError>> {
    let mut bytes = Vec::new();
    let mut token_iter = tokens.iter().filter(|x| x.t != TokenType::Comment && x.t != TokenType::Whitespace).peekable();
    let mut errors = Vec::new();
    let mut aliases: HashMap<(AssemblyMode, String), RegisterName> = HashMap::new();
    let mut entry_found = false;
    let mut assembly_mode = None;

    while let Some(token) = token_iter.next() {
        match token.t {
            TokenType::Whitespace |
            TokenType::Comment => {},
            TokenType::Command(command_t) => {
                println!("assembling command: {:?}", command_t);
                match command_t {
                    CommandType::Entry => {
                        let current_assembly_mode = match assembly_mode {
                            Some(mode) => mode,
                            None => {
                                errors.push(SourceError {
                                    message: format!("entry! used before an assembler mode was set (vertex!, fragment!, or compute!)"),
                                    line: token.line,
                                    column: token.column,
                                });
                                break;
                            }
                        };
                        if !entry_found {
                            if entry_mode == current_assembly_mode {
                                entry_found = true;
                                bytes.clear();
                            }
                        } else {
                            break;
                        }
                    },
                    CommandType::Alias => {
                        let assembly_mode = match assembly_mode {
                            Some(mode) => mode,
                            None => {
                                errors.push(SourceError {
                                    message: format!("alias! used before an assembler mode was set (vertex!, fragment!, or compute!)"),
                                    line: token.line,
                                    column: token.column,
                                });
                                break;
                            }
                        };
                        if let Err(error) = handle_alias(&mut bytes, &mut aliases, &token, &mut token_iter, assembly_mode) {
                            errors.push(error);
                        }
                    },
                    CommandType::SetMode(new_asembly_mode) => {
                        assembly_mode = Some(new_asembly_mode);
                    }
                    _ => todo!()
                }
            },
            TokenType::Instruction(instruction_t) => {
                if let Err(error) = handle_instruction(&mut bytes, instruction_t, token, &mut token_iter, &aliases, assembly_mode) {
                    errors.push(error);
                }
            },
            TokenType::Name => {
                errors.push(SourceError {
                    message: format!("Unexpected token: '{}'", token.value.clone().unwrap_or("".to_owned())),
                    line: token.line,
                    column: token.column,
                })
            },
            TokenType::Number => {
                errors.push(SourceError {
                    message: format!("Unexpected numer"),
                    line: token.line,
                    column: token.column,
                })
            },
            TokenType::Comma => {
                errors.push(SourceError {
                    message: format!("Unexpected comma"),
                    line: token.line,
                    column: token.column,
                })
            },
            TokenType::Dot => {
                errors.push(SourceError {
                    message: format!("Unexpected dot"),
                    line: token.line,
                    column: token.column,
                })
            },
            TokenType::Register(_) => {
                errors.push(SourceError {
                    message: format!("Unexpected register name"),
                    line: token.line,
                    column: token.column,
                })
            },
            TokenType::Colon => {
                errors.push(SourceError {
                    message: format!("Unexpected colon"),
                    line: token.line,
                    column: token.column,
                })
            },
        }
    }
    if !entry_found {
        errors.push(SourceError {
            message: format!("No matching shader entry point in file"),
            line: 0,
            column: 0,
        })
    }
    if errors.len() != 0 {
        return Err(errors)
    } else {
        return Ok(bytes.into_boxed_slice())
    }
}

fn handle_alias<'t, I: Iterator<Item = &'t Token>>(_bytes: &mut Vec<u8>, aliases: &mut HashMap<(AssemblyMode, String), RegisterName>, alias_token: &'t Token, iter: &mut Peekable<I>, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
    let name_token = expect_token(TokenType::Name, iter)?;
    let alias_name = name_token.value.clone().expect("expected Name token to have a value");
    expect_token(TokenType::Colon, iter)?;
    let register_token = iter.next().ok_or_else(|| SourceError {
        message: format!("Unexpected end of file"),
        line: alias_token.line,
        column: alias_token.column
    })?;
    let register_name = match register_token.t {
        TokenType::Register(register_name) => register_name,
        _ => Err(SourceError {
            message: format!("Expected register name after colon in alias declaration"),
            line: register_token.line,
            column: register_token.column,
        })?
    };
    println!("new aliases: \"{}\": {:?}", alias_name, register_name);
    aliases.insert((assembly_mode, alias_name), register_name);
    Ok(())
}

fn handle_instruction<'t, I: Iterator<Item = &'t Token>>(bytes: &mut Vec<u8>, instruction_t: InstructionType, instruction_token: &'t Token, iter: &mut Peekable<I>, aliases: &HashMap<(AssemblyMode, String), RegisterName>, assembly_mode: Option<AssemblyMode>) -> Result<(), SourceError> {
    let assembly_mode = assembly_mode.ok_or_else(|| 
        SourceError {
            message: format!("instruction before assembly mode command (vertex, fragment, compute)"),
            line: instruction_token.line,
            column: instruction_token.column
        }
    )?;
    match instruction_t {
        InstructionType::Push => {
            let (src, src_token) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            write_push(bytes, src, src_token, assembly_mode)?;
        },
        InstructionType::Pop => {
            let (dst, dst_token) = expect_write_register(iter, Some(aliases), assembly_mode)?;
            write_pop(bytes, dst, dst_token, assembly_mode)?;
        },
        InstructionType::Mov => {
            let dst = expect_write_register(iter, Some(aliases), assembly_mode)?;
            let dst_component = if let Some(_dot_token) = try_expect_token(TokenType::Dot, iter) {
                Some(expect_token(TokenType::Name, iter)?)
            } else {
                None
            };
            expect_token(TokenType::Comma, iter)?;
            let src = expect_read_register(iter, Some(aliases), assembly_mode)?;
            let src_component = if let Some(_dot_token) = try_expect_token(TokenType::Dot, iter) {
                Some(expect_token(TokenType::Name, iter)?)
            } else {
                None
            };
            write_mov(bytes, instruction_token, dst.0, dst_component, src.0, src_component, assembly_mode)?;
        },
        InstructionType::CMov => {
            let cond = expect_read_register(iter, Some(aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let dst = expect_write_register(iter, Some(aliases), assembly_mode)?;
            let dst_component = if let Some(_dot_token) = try_expect_token(TokenType::Dot, iter) {
                Some(expect_token(TokenType::Name, iter)?)
            } else {
                None
            };
            expect_token(TokenType::Comma, iter)?;
            let src = expect_read_register(iter, Some(aliases), assembly_mode)?;
            let src_component = if let Some(_dot_token) = try_expect_token(TokenType::Dot, iter) {
                Some(expect_token(TokenType::Name, iter)?)
            } else {
                None
            };
            write_cmov(bytes, instruction_token, cond.0, dst.0, dst_component, src.0, src_component, assembly_mode)?;
        },
        InstructionType::Read => {
            todo!()
        },
        InstructionType::Write => todo!(),
        InstructionType::CRead => todo!(),
        InstructionType::CWrite => todo!(),
        InstructionType::Load => todo!(),
        InstructionType::Store => todo!(),
        InstructionType::Discard => todo!(),
        InstructionType::Conv => todo!(),
        InstructionType::Neg(_)  => todo!(),
        InstructionType::Sign(_) => todo!(),
        InstructionType::Recip => todo!(),
        InstructionType::Sin => todo!(),
        InstructionType::Cos => todo!(),
        InstructionType::Tan => todo!(),
        InstructionType::ASin => todo!(),
        InstructionType::ACos => todo!(),
        InstructionType::Atan => todo!(),
        InstructionType::Ln => todo!(),
        InstructionType::Exp => todo!(),
        InstructionType::Cmp(comparison) => {
            let (dst, _) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a, _) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (b, _) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            write_cmp(bytes, instruction_token, dst, a, b, comparison, assembly_mode)?;
        },
        InstructionType::UCmp(comparison) => {
            let (dst, _) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a, _) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (b, _) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            write_ucmp(bytes, instruction_token, dst, a, b, comparison, assembly_mode)?;
        },
        InstructionType::FCmp(comparison) => {
            let (dst, _) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a, _) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (b, _) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            write_fcmp(bytes, instruction_token, dst, a, b, comparison, assembly_mode)?;
        },
        InstructionType::Add(_) |
        InstructionType::Sub(_) |
        InstructionType::Mul(_) |
        InstructionType::Div(_) |
        InstructionType::Mod(_) => {
            let (dst, _) = expect_write_register(iter, Some(aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a, _) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (b, _) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            write_scalar_binary_op(bytes, &instruction_token, instruction_t, dst, a, b, assembly_mode)?;
        },
        InstructionType::Atan2 => todo!(),
        InstructionType::And => todo!(),
        InstructionType::AndN => todo!(),
        InstructionType::Or => todo!(),
        InstructionType::Xor => todo!(),
        InstructionType::Fma(_) => todo!(),
        InstructionType::Lerp => todo!(),
        InstructionType::Norm => todo!(),
        InstructionType::Mag => todo!(),
        InstructionType::Cross => todo!(),
        InstructionType::MatrixMultiply4x4V4 => {
            let (dst, _) = expect_write_register(iter, Some(aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a0, _) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a1, _) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a2, _) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a3, _) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (x, _) = expect_read_register(iter, Some(aliases), assembly_mode)?;
            write_mul_m44_v4(bytes, &instruction_token, dst, a0, a1, a2, a3, x, assembly_mode)?;
        }
    }
    Ok(())
}