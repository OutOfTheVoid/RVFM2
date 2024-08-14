use std::collections::HashMap;
use std::iter::Peekable;

use super::lexer::*;
use super::src_error::*;
use super::instructions::*;

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

pub fn try_expect_token_with<'t, I: Iterator<Item = &'t Token>>(test: impl Fn(&TokenType) -> bool, iter: &mut Peekable<I>) -> Option<&'t Token> {
    match iter.peek() {
        Some(token) => {
            if test(&token.t) {
                Some(iter.next().unwrap())
            } else {
                None
            }
        },
        None => None
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

pub fn expect_buffer<'t, I: Iterator<Item = &'t Token>>(iter: &mut Peekable<I>, aliases: Option<&HashMap<String, u8>>, assembly_mode: AssemblyMode) -> Result<(u8, &'t Token), SourceError> {
    let token = expect_token_with(&format!("register readable from {:?} shader", assembly_mode),
        |t| match t {
                TokenType::Buffer(_) => true,
                TokenType::Name => aliases.is_some(),
                _ => false,
            },
        iter)?;
    if let Token { t: TokenType::Buffer(n), .. } = token {
        Ok((*n, token))
    } else if let Token { t: TokenType::Name, value: Some(name), line, column } = token {
        if let Some(aliases) = aliases {
            if let Some(buffer) = aliases.get(name) {
                Ok((*buffer, token))
            } else {
                Err(SourceError {
                    message: format!("No buffer with the name {}", name),
                    line: *line,
                    column: *column
                })
            }
        } else {
            Err(SourceError {
                message: format!("\"{}\" is not a valid buffer name", name),
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
    let mut register_aliases: HashMap<(AssemblyMode, String), RegisterName> = HashMap::new();
    let mut buffer_aliases: HashMap<String, u8> = HashMap::new();
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
                        if let Err(error) = handle_alias(&mut bytes, &mut register_aliases, &token, &mut token_iter, assembly_mode) {
                            errors.push(error);
                        }
                    },
                    CommandType::BufferDecl => {
                        if let Err(error) = handle_buffer_alias(&mut bytes, &mut buffer_aliases, &token, &mut token_iter) {
                            errors.push(error);
                        }
                    }
                    CommandType::SetMode(new_asembly_mode) => {
                        assembly_mode = Some(new_asembly_mode);
                    }
                    _ => todo!()
                }
            },
            TokenType::Instruction(instruction_t) => {
                if let Err(error) = handle_instruction(&mut bytes, instruction_t, token, &mut token_iter, &register_aliases, &buffer_aliases, assembly_mode) {
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
            TokenType::Buffer(_) => {
                errors.push(SourceError {
                    message: format!("Unexpected buffer name"),
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

fn handle_buffer_alias<'t, I: Iterator<Item = &'t Token>>(_bytes: &mut Vec<u8>, aliases: &mut HashMap<String, u8>, alias_token: &'t Token, iter: &mut Peekable<I>) -> Result<(), SourceError> {
    let name_token = expect_token(TokenType::Name, iter)?;
    let alias_name = name_token.value.clone().expect("expected Name token to have a value");
    expect_token(TokenType::Colon, iter)?;
    let buffer_token = iter.next().ok_or_else(|| SourceError {
        message: format!("Unexpected end of file"),
        line: alias_token.line,
        column: alias_token.column
    })?;
    let buffer_name = match buffer_token.t {
        TokenType::Buffer(buffer_name) => buffer_name,
        _ => Err(SourceError {
            message: format!("Expected register name after colon in alias declaration"),
            line: buffer_token.line,
            column: buffer_token.column,
        })?
    };
    println!("new buffer aliases: \"{}\": {:?}", alias_name, buffer_name);
    aliases.insert(alias_name, buffer_name);
    Ok(())
}

fn handle_instruction<'t, I: Iterator<Item = &'t Token>>(bytes: &mut Vec<u8>, instruction_t: InstructionType, instruction_token: &'t Token, iter: &mut Peekable<I>, register_aliases: &HashMap<(AssemblyMode, String), RegisterName>, buffer_aliases: &HashMap<String, u8>, assembly_mode: Option<AssemblyMode>) -> Result<(), SourceError> {
    let assembly_mode = assembly_mode.ok_or_else(|| 
        SourceError {
            message: format!("instruction before assembly mode command (vertex, fragment, compute)"),
            line: instruction_token.line,
            column: instruction_token.column
        }
    )?;
    match instruction_t {
        InstructionType::Push => {
            let (src, src_token) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            write_push(bytes, src, src_token, assembly_mode)?;
        },
        InstructionType::Pop => {
            let (dst, dst_token) = expect_write_register(iter, Some(register_aliases), assembly_mode)?;
            write_pop(bytes, dst, dst_token, assembly_mode)?;
        },
        InstructionType::Mov => {
            let dst = expect_write_register(iter, Some(register_aliases), assembly_mode)?;
            let dst_component = if let Some(_dot_token) = try_expect_token(TokenType::Dot, iter) {
                Some(expect_token(TokenType::Name, iter)?)
            } else {
                None
            };
            expect_token(TokenType::Comma, iter)?;
            let src = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            let src_component = if let Some(_dot_token) = try_expect_token(TokenType::Dot, iter) {
                Some(expect_token(TokenType::Name, iter)?)
            } else {
                None
            };
            write_mov(bytes, instruction_token, dst.0, dst_component, src.0, src_component, assembly_mode)?;
        },
        InstructionType::CMov => {
            let cond = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let dst = expect_write_register(iter, Some(register_aliases), assembly_mode)?;
            let dst_component = if let Some(_dot_token) = try_expect_token(TokenType::Dot, iter) {
                Some(expect_token(TokenType::Name, iter)?)
            } else {
                None
            };
            expect_token(TokenType::Comma, iter)?;
            let src = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            let src_component = if let Some(_dot_token) = try_expect_token(TokenType::Dot, iter) {
                Some(expect_token(TokenType::Name, iter)?)
            } else {
                None
            };
            write_cmov(bytes, instruction_token, cond.0, dst.0, dst_component, src.0, src_component, assembly_mode)?;
        },

        InstructionType::Read(dt)       => {
            let (dst, dst_token) = expect_write_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (buffer, _) = expect_buffer(iter, Some(buffer_aliases), assembly_mode)?;
            let offset_token = expect_token(TokenType::Number, iter)?;
            let offset = if let NumberParse::Integer(x) = parse_number(offset_token) {
                x as i32 as u32
            } else {
                Err(SourceError {
                    message: format!("Invalid offset value - floating point values aren't allowed as offsets"),
                    line: offset_token.line,
                    column: offset_token.column
                })?
            };
            let src_addr_reg = if try_expect_token(TokenType::Comma, iter).is_some() {
                let (src_addr_reg, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
                Some(src_addr_reg)
            } else {
                None
            };
            write_buffer_read_op(bytes, &instruction_token, instruction_t, dt, dst, dst_token, src_addr_reg, offset, buffer, assembly_mode)?;
        },
        InstructionType::Write(_)        => todo!(),
        InstructionType::CRead           => todo!(),
        InstructionType::CWrite          => todo!(),
        InstructionType::Store           => todo!(),

        InstructionType::Neg(_)          |
        InstructionType::Sign(_)         |
        InstructionType::Recip           |
        InstructionType::Sin             |
        InstructionType::Cos             |
        InstructionType::Tan             |
        InstructionType::ASin            |
        InstructionType::ACos            |
        InstructionType::Atan            |
        InstructionType::Ln              |
        InstructionType::Exp             |
        InstructionType::ConvertF32ToI32 |
        InstructionType::ConvertF32ToU32 | 
        InstructionType::ConvertI32ToF32 |
        InstructionType::ConvertU32ToF32 |
        InstructionType::Norm2           |
        InstructionType::Norm3           |
        InstructionType::Norm4           |
        InstructionType::Mag2            |
        InstructionType::Mag3            |
        InstructionType::Mag4            |
        InstructionType::SqMag2          |
        InstructionType::SqMag3          |
        InstructionType::SqMag4          => {
            let (dst, _) = expect_write_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (src, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            write_unary_op(bytes, &instruction_token, instruction_t, dst, src, assembly_mode)?;
        },

        InstructionType::Cmp(comparison) => {
            let (dst, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (b, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            write_cmp(bytes, instruction_token, dst, a, b, comparison, assembly_mode)?;
        },
        InstructionType::UCmp(comparison) => {
            let (dst, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (b, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            write_ucmp(bytes, instruction_token, dst, a, b, comparison, assembly_mode)?;
        },
        InstructionType::FCmp(comparison) => {
            let (dst, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (b, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            write_fcmp(bytes, instruction_token, dst, a, b, comparison, assembly_mode)?;
        },

        InstructionType::Atan2  |
        InstructionType::And    |
        InstructionType::AndN   |
        InstructionType::Or     |
        InstructionType::Xor    |
        InstructionType::Add(_) |
        InstructionType::Sub(_) |
        InstructionType::Mul(_) |
        InstructionType::Div(_) |
        InstructionType::Mod(_) |
        InstructionType::Cross => {
            let (dst, _) = expect_write_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (b, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            write_binary_op(bytes, &instruction_token, instruction_t, dst, a, b, assembly_mode)?;
        },
        
        InstructionType::Lerp |
        InstructionType::Fma(_) => {
            let (dst, _) = expect_write_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (b, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (c, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            write_ternary_op(bytes, &instruction_token, instruction_t, dst, a, b, c, assembly_mode)?;
        },

        InstructionType::MatrixMultiply4x4V4 => {
            let (dst, _) = expect_write_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a0, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a1, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a2, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (a3, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            expect_token(TokenType::Comma, iter)?;
            let (x, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
            write_mul_m44_v4(bytes, &instruction_token, dst, a0, a1, a2, a3, x, assembly_mode)?;
        },

        InstructionType::Read(buffer_type) => {
            let (dst, _) = expect_read_register(iter, Some(register_aliases), assembly_mode)?;
        }

        InstructionType::Load(op_type, data_type) => {
            todo!()
        },


    }
    Ok(())
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
enum NumberParseMode {
    Empty,
    Decimal,
    ZeroStart,
    HexadecimalStart,
    Hexadecimal,
    OctalStart,
    Octal,
    BinaryStart,
    Binary,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
enum NumberParse {
    Integer(i64),
    Float(u32) // f32 bits
}

fn parse_number(token: &Token) -> NumberParse {
    // todo: parse floats, negative numbers
    let mut chars = match &token.value {
        Some(value) => value.chars().peekable(),
        None => panic!("Number token passed to parse_number without a value! (internal error)"),
    };
    let mut parse_mode = NumberParseMode::Empty;
    let mut value = 0i64;
    while let Some(c) = chars.next() {
        parse_mode = match (parse_mode, c) {
            (NumberParseMode::Empty, '0') => {
                NumberParseMode::ZeroStart
            },
            (NumberParseMode::Empty, '1'..='9') => {
                value = c as i64 - '0' as i64;
                NumberParseMode::Decimal
            },

            (NumberParseMode::ZeroStart, '0'..='9') |
            (NumberParseMode::Decimal,   '0'..='9') => {
                value *= 10;
                value = c as i64 - '0' as i64;
                NumberParseMode::Decimal
            },

            (NumberParseMode::ZeroStart, 'x') |
            (NumberParseMode::ZeroStart, 'X') => {
                NumberParseMode::HexadecimalStart
            },

            (NumberParseMode::ZeroStart, 'o') |
            (NumberParseMode::ZeroStart, 'O') => {
                NumberParseMode::OctalStart
            },

            (NumberParseMode::ZeroStart, 'b') |
            (NumberParseMode::ZeroStart, 'B') => {
                NumberParseMode::BinaryStart
            },

            (NumberParseMode::HexadecimalStart, '0'..='9') |
            (NumberParseMode::Hexadecimal,      '0'..='9') => {
                value <<= 4;
                value += c as i64 - '0' as i64;
                NumberParseMode::Hexadecimal
            },

            (NumberParseMode::HexadecimalStart, 'a'..='f') |
            (NumberParseMode::Hexadecimal,      'a'..='f') => {
                value <<= 4;
                value += c as i64 - 'a' as i64 + 10i64;
                NumberParseMode::Hexadecimal
            },

            (NumberParseMode::HexadecimalStart, 'A'..='F') |
            (NumberParseMode::Hexadecimal,      'A'..='F') => {
                value <<= 4;
                value += c as i64 - 'A' as i64 + 10i64;
                NumberParseMode::Hexadecimal
            },

            (NumberParseMode::OctalStart, '0'..='7') |
            (NumberParseMode::Octal,      '0'..='7') => {
                value <<= 3;
                value += c as i64 - '0' as i64;
                NumberParseMode::Octal
            },

            (NumberParseMode::BinaryStart, '0'..='1') |
            (NumberParseMode::Binary,      '0'..='1') => {
                value <<= 1;
                value += c as i64 - '0' as i64;
                NumberParseMode::Octal
            },

            _ => panic!("parse_number(): Invalid value in token (internal error)"),
        };
    }
    match parse_mode {
        NumberParseMode::Empty |
        NumberParseMode::BinaryStart |
        NumberParseMode::HexadecimalStart |
        NumberParseMode::OctalStart => panic!("parse_number(): Invalid value in token (internal error)"),
        
        NumberParseMode::Decimal |
        NumberParseMode::Binary |
        NumberParseMode::Hexadecimal |
        NumberParseMode::Octal => NumberParse::Integer(value),

        NumberParseMode::ZeroStart => NumberParse::Integer(0),
    }
    
}