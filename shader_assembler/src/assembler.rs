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

fn expect_write_register<'t, I: Iterator<Item = &'t Token>>(iter: &mut Peekable<I>, entry_type: EntryType) -> Result<(RegisterName, &'t Token), SourceError> {
    let token = expect_token_with(&format!("register writable from shader with {:?}", entry_type),
        |t| match t {
                TokenType::Register(RegisterName::LocalS(_)) |
                TokenType::Register(RegisterName::LocalV(_)) |
                TokenType::Register(RegisterName::OutputS(_)) |
                TokenType::Register(RegisterName::OutputV(_)) => true,
                TokenType::Register(RegisterName::BuiltinS(scalar_builtin)) => {
                    match (entry_type, scalar_builtin) {
                        (EntryType::Fragment, ScalarBuiltin::Depth) => true,
                        _ => false,
                    }
                },
                TokenType::Register(RegisterName::BuiltinV(vector_builtin)) => {
                    match (entry_type, vector_builtin) {
                        (EntryType::Vertex, VectorBuiltin::VertexPosition) => true,
                        _ => false,
                    }
                },
                _ => false,
            },
        iter)?;
    if let Token { t: TokenType::Register(name), .. } = token {
        Ok((*name, token))
    } else {
        unreachable!()
    }
}

fn expect_read_register<'t, I: Iterator<Item = &'t Token>>(iter: &mut Peekable<I>, entry_type: EntryType) -> Result<(RegisterName, &'t Token), SourceError> {
    let token = expect_token_with(&format!("register readable from shader with {:?}", entry_type),
        |t| match t {
                TokenType::Register(RegisterName::ConstantS(_)) |
                TokenType::Register(RegisterName::ConstantV(_)) |
                TokenType::Register(RegisterName::LocalS(_)) |
                TokenType::Register(RegisterName::LocalV(_)) |
                TokenType::Register(RegisterName::InputS(_)) |
                TokenType::Register(RegisterName::InputV(_)) => true,
                TokenType::Register(RegisterName::BuiltinS(scalar_builtin)) => {
                    match (entry_type, scalar_builtin) {
                        (EntryType::Vertex, ScalarBuiltin::VertexId)        |
                        (EntryType::Vertex, ScalarBuiltin::ProvokingVertex) |
                        (EntryType::Fragment, ScalarBuiltin::Depth)         => true,
                        _ => false,
                    }
                },
                TokenType::Register(RegisterName::BuiltinV(vector_builtin)) => {
                    match (entry_type, vector_builtin) {
                        (EntryType::Fragment, VectorBuiltin::VertexPosition) |
                        (EntryType::Fragment, VectorBuiltin::Barycentric   ) => true,
                        _ => false
                    }
                }
                _ => false,
            },
        iter)?;
    if let Token { t: TokenType::Register(name), .. } = token {
        Ok((*name, token))
    } else {
        unreachable!()
    }
}

pub fn run_assembler(tokens: &[Token], entry_type: EntryType) -> Result<Box<[u8]>, Vec<SourceError>> {
    let mut bytes = Vec::new();
    let mut token_iter = tokens.iter().filter(|x| x.t != TokenType::Comment && x.t != TokenType::Whitespace).peekable();
    let mut errors = Vec::new();
    while let Some(token) = token_iter.next() {
        if token.t == TokenType::Command(CommandType::Entry(entry_type)) {
            break;
        }
    }
    while let Some(token) = token_iter.next() {
        match token.t {
            TokenType::Whitespace |
            TokenType::Comment => {},
            TokenType::Command(command_t) => {
                match command_t {
                    CommandType::Entry(_) => {
                        break;
                    }
                    _ => todo!()
                }
            },
            TokenType::Instruction(instruction_t) => {
                if let Err(error) = handle_instruction(&mut bytes, instruction_t, token,&mut token_iter, entry_type) {
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
        }
    }
    if errors.len() != 0 {
        return Err(errors)
    } else {
        return Ok(bytes.into_boxed_slice())
    }
}

fn handle_instruction<'t, I: Iterator<Item = &'t Token>>(bytes: &mut Vec<u8>, instruction_t: InstructionType, instruction_token: &'t Token, iter: &mut Peekable<I>, entry_type: EntryType) -> Result<(), SourceError> {
    match instruction_t {
        InstructionType::Push => {
            let (src, src_token) = expect_read_register(iter, entry_type)?;
            write_push(bytes, src, src_token, entry_type)?;
        },
        InstructionType::Pop => {
            let (dst, dst_token) = expect_write_register(iter, entry_type)?;
            write_pop(bytes, dst, dst_token, entry_type)?;
        },
        InstructionType::Mov => {
            let dst = expect_write_register(iter, entry_type)?;
            let dst_component = if let Some(_dot_token) = try_expect_token(TokenType::Dot, iter) {
                Some(expect_token(TokenType::Name, iter)?)
            } else {
                None
            };
            expect_token(TokenType::Comma, iter)?;
            let src = expect_read_register(iter, entry_type)?;
            let src_component = if let Some(_dot_token) = try_expect_token(TokenType::Dot, iter) {
                Some(expect_token(TokenType::Name, iter)?)
            } else {
                None
            };
            write_mov(bytes, instruction_token, dst.0, dst_component, src.0, src_component, entry_type)?;
        },
        InstructionType::CMov => {
            let cond = expect_read_register(iter, entry_type)?;
            let dst = expect_write_register(iter, entry_type)?;
            let dst_component = if let Some(_dot_token) = try_expect_token(TokenType::Dot, iter) {
                Some(expect_token(TokenType::Name, iter)?)
            } else {
                None
            };
            expect_token(TokenType::Comma, iter)?;
            let src = expect_read_register(iter, entry_type)?;
            let src_component = if let Some(_dot_token) = try_expect_token(TokenType::Dot, iter) {
                Some(expect_token(TokenType::Name, iter)?)
            } else {
                None
            };
            write_cmov(bytes, instruction_token, cond.0, dst.0, dst_component, src.0, src_component, entry_type)?;
        },
        InstructionType::Read => todo!(),
        InstructionType::Write => todo!(),
        InstructionType::CRead => todo!(),
        InstructionType::CWrite => todo!(),
        InstructionType::Load => todo!(),
        InstructionType::Store => todo!(),
        InstructionType::Discard => todo!(),
        InstructionType::Conv => todo!(),
        InstructionType::Neg => todo!(),
        InstructionType::Sign => todo!(),
        InstructionType::Recip => todo!(),
        InstructionType::Sin => todo!(),
        InstructionType::Cos => todo!(),
        InstructionType::Tan => todo!(),
        InstructionType::ASin => todo!(),
        InstructionType::ACos => todo!(),
        InstructionType::Atan => todo!(),
        InstructionType::Ln => todo!(),
        InstructionType::Exp => todo!(),
        InstructionType::Cmp(_) => todo!(),
        InstructionType::Add => todo!(),
        InstructionType::Sub => todo!(),
        InstructionType::Mul => todo!(),
        InstructionType::Div => todo!(),
        InstructionType::Mod => todo!(),
        InstructionType::Atan2 => todo!(),
        InstructionType::UCmp(_) => todo!(),
        InstructionType::And => todo!(),
        InstructionType::AndN => todo!(),
        InstructionType::Or => todo!(),
        InstructionType::Xor => todo!(),
        InstructionType::Fma => todo!(),
        InstructionType::Lerp => todo!(),
        InstructionType::Norm => todo!(),
        InstructionType::Mag => todo!(),
        InstructionType::Cross => todo!(),
    }
    Ok(())
}