use super::src_error::*;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum EntryType {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum CommandType {
    Entry(EntryType),
    Var,
    TextureDecl,
    BufferDecl,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Comparison {
    Eq,
    Neq,
    Lt,
    Lte
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum InstructionType {
    Push,
    Pop,
    Mov,
    CMov,
    Read,
    Write,
    CRead,
    CWrite,
    Load,
    Store,
    Discard,
    Conv,
    Neg,
    Sign,
    Recip,
    Sin,
    Cos,
    Tan,
    ASin,
    ACos,
    Atan,
    Ln,
    Exp,
    Cmp(Comparison),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Atan2,
    UCmp(Comparison),
    And,
    AndN,
    Or,
    Xor,
    Fma,
    Lerp,
    Norm,
    Mag,
    Cross,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum VectorBuiltin {
    VertexPosition,
    Barycentric,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ScalarBuiltin {
    Depth,
    VertexId,
    ProvokingVertex,
    Discard,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum RegisterName {
    LocalS(u8),
    LocalV(u8),
    InputS(u8),
    InputV(u8),
    BuiltinS(ScalarBuiltin),
    BuiltinV(VectorBuiltin),
    ConstantS(u8),
    ConstantV(u8),
    OutputS(u8),
    OutputV(u8),
}

impl RegisterName {
    pub fn is_vector(&self) -> bool {
        match self {
            Self::LocalV(_) |
            Self::InputV(_) |
            Self::BuiltinV(_) |
            Self::ConstantV(_) |
            Self::OutputV(_) => true,
            _ => false
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TokenType {
    Whitespace,
    Name,
    Register(RegisterName),
    Command(CommandType),
    Instruction(InstructionType),
    Number,
    Comment,
    Comma,
    Dot,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub t: TokenType,
    pub line: usize,
    pub column: usize,
    pub value: Option<String>,
}

pub fn run_lexer(input: &str) -> Result<Vec<Token>, Vec<SourceError>>  {
    let mut line = 1;
    let mut column = 1;
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut chars = input.chars().peekable();
    let mut value_string = String::new();
    while let Some(c) = chars.next() {
        value_string = "".into();
        let token_line = line;
        let token_column = column;
        column += 1;
        match c {
            ' ' | '\t' | '\n' => {
                if c == '\n' {
                    column = 1;
                    line += 1;
                }
                while let Some(&c) = chars.peek() {
                    match c {
                        ' ' | '\t' | '\n' => {
                            chars.next();
                            column += 1;
                            if c == '\n' {
                                column = 1;
                                line += 1;
                            }
                        },
                        _ => break
                    }
                }
                tokens.push(Token {
                    t: TokenType::Whitespace,
                    line: token_line,
                    column: token_column,
                    value: None,
                });
            },

            ',' => {
                tokens.push(Token {
                    t: TokenType::Comma,
                    line: token_line,
                    column: token_column,
                    value: None,
                });
            }

            '.' => {
                tokens.push(Token {
                    t: TokenType::Comma,
                    line: token_line,
                    column: token_column,
                    value: None,
                });
            }

            'a' ..= 'z' | 'A' ..= 'Z' | '_' => {
                value_string.push(c);
                while let Some(&c) = chars.peek() {
                    match c {
                        'a' ..= 'z' | 'A' ..= 'Z' | '_' | '0' ..= '9' => {
                            value_string.push(c);
                            chars.next();
                            column += 1;
                        },
                        _ => break,
                    }
                }
                let t = if value_string.to_lowercase().starts_with("vloc_") {
                    if let Ok(index) = value_string[5..].parse::<u32>() {
                        TokenType::Register(RegisterName::LocalV(index as u8))
                    } else {
                        errors.push(SourceError {
                            message: format!("Unexpected character '{}'", c),
                            line: token_line,
                            column: token_column,
                        });
                        continue;
                    }
                } else if value_string.to_lowercase().starts_with("sloc_") {
                    if let Ok(index) = value_string[5..].parse::<u32>() {
                        TokenType::Register(RegisterName::LocalS(index as u8))
                    } else {
                        errors.push(SourceError {
                            message: format!("Unexpected character '{}'", c),
                            line: token_line,
                            column: token_column,
                        });
                        continue;
                    }
                } else if value_string.to_lowercase().starts_with("vin_") {
                    if let Ok(index) = value_string[4..].parse::<u32>() {
                        TokenType::Register(RegisterName::InputV(index as u8))
                    } else {
                        errors.push(SourceError {
                            message: format!("Unexpected character '{}'", c),
                            line: token_line,
                            column: token_column,
                        });
                        continue;
                    }
                } else if value_string.to_lowercase().starts_with("sin_") {
                    if let Ok(index) = value_string[4..].parse::<u32>() {
                        TokenType::Register(RegisterName::InputS(index as u8))
                    } else {
                        errors.push(SourceError {
                            message: format!("Unexpected character '{}'", c),
                            line: token_line,
                            column: token_column,
                        });
                        continue;
                    }
                } else if value_string.to_lowercase().starts_with("vout_") {
                    if let Ok(index) = value_string[5..].parse::<u32>() {
                        TokenType::Register(RegisterName::OutputV(index as u8))
                    } else {
                        errors.push(SourceError {
                            message: format!("Unexpected character '{}'", c),
                            line: token_line,
                            column: token_column,
                        });
                        continue;
                    }
                } else if value_string.to_lowercase().starts_with("sout_") {
                    if let Ok(index) = value_string[5..].parse::<u32>() {
                        TokenType::Register(RegisterName::OutputS(index as u8))
                    } else {
                        errors.push(SourceError {
                            message: format!("Unexpected character '{}'", c),
                            line: token_line,
                            column: token_column,
                        });
                        continue;
                    }
                } else if value_string.to_lowercase().starts_with("vconst_") {
                    if let Ok(index) = value_string[7..].parse::<u32>() {
                        TokenType::Register(RegisterName::ConstantV(index as u8))
                    } else {
                        errors.push(SourceError {
                            message: format!("Unexpected character '{}'", c),
                            line: token_line,
                            column: token_column,
                        });
                        continue;
                    }
                } else if value_string.to_lowercase().starts_with("sconst_") {
                    if let Ok(index) = value_string[7..].parse::<u32>() {
                        TokenType::Register(RegisterName::ConstantS(index as u8))
                    } else {
                        errors.push(SourceError {
                            message: format!("Unexpected character '{}'", c),
                            line: token_line,
                            column: token_column,
                        });
                        continue;
                    }
                } else {
                    match value_string.to_lowercase().as_str() {
                        "vertex_position"  => TokenType::Register(RegisterName::BuiltinV(VectorBuiltin::VertexPosition)),
                        "barycentric"      => TokenType::Register(RegisterName::BuiltinV(VectorBuiltin::Barycentric)),
                        "depth"            => TokenType::Register(RegisterName::BuiltinS(ScalarBuiltin::Depth)),
                        "vertex_id"        => TokenType::Register(RegisterName::BuiltinS(ScalarBuiltin::VertexId)),
                        "provoking_vertex" => TokenType::Register(RegisterName::BuiltinS(ScalarBuiltin::ProvokingVertex)),
                        "entry_v"          => TokenType::Command(CommandType::Entry(EntryType::Vertex)),
                        "entry_f"          => TokenType::Command(CommandType::Entry(EntryType::Fragment)),
                        "entry_c"          => TokenType::Command(CommandType::Entry(EntryType::Compute)),
                        "var"              => TokenType::Command(CommandType::Var),
                        "texture"          => TokenType::Command(CommandType::TextureDecl),
                        "buffer"           => TokenType::Command(CommandType::BufferDecl),
                        "push"             => TokenType::Instruction(InstructionType::Push),
                        "pop"              => TokenType::Instruction(InstructionType::Pop),
                        "mov"              => TokenType::Instruction(InstructionType::Mov),
                        "cmov"             => TokenType::Instruction(InstructionType::CMov),
                        "read"             => TokenType::Instruction(InstructionType::Read),
                        "write"            => TokenType::Instruction(InstructionType::Write),
                        "cread"            => TokenType::Instruction(InstructionType::CRead),
                        "cwrite"           => TokenType::Instruction(InstructionType::CWrite),
                        "load"             => TokenType::Instruction(InstructionType::Load),
                        "store"            => TokenType::Instruction(InstructionType::Store),
                        "discard"          => TokenType::Instruction(InstructionType::Discard),
                        "conv"             => TokenType::Instruction(InstructionType::Conv),
                        "neg"              => TokenType::Instruction(InstructionType::Neg),
                        "sign"             => TokenType::Instruction(InstructionType::Sign),
                        "recip"            => TokenType::Instruction(InstructionType::Recip),
                        "sin"              => TokenType::Instruction(InstructionType::Sin),
                        "cos"              => TokenType::Instruction(InstructionType::Cos),
                        "tan"              => TokenType::Instruction(InstructionType::Tan),
                        "asin"             => TokenType::Instruction(InstructionType::ASin),
                        "acos"             => TokenType::Instruction(InstructionType::ACos),
                        "atan"             => TokenType::Instruction(InstructionType::Atan),
                        "ln"               => TokenType::Instruction(InstructionType::Ln),
                        "exp"              => TokenType::Instruction(InstructionType::Exp),
                        "cmp_eq"           => TokenType::Instruction(InstructionType::Cmp(Comparison::Eq)),
                        "cmp_neq"          => TokenType::Instruction(InstructionType::Cmp(Comparison::Neq)),
                        "cmp_lt"           => TokenType::Instruction(InstructionType::Cmp(Comparison::Lt)),
                        "cmp_lte"          => TokenType::Instruction(InstructionType::Cmp(Comparison::Lte)),
                        "add"              => TokenType::Instruction(InstructionType::Add),
                        "sub"              => TokenType::Instruction(InstructionType::Sub),
                        "mul"              => TokenType::Instruction(InstructionType::Mul),
                        "div"              => TokenType::Instruction(InstructionType::Div),
                        "mod"              => TokenType::Instruction(InstructionType::Mod),
                        "atan2"            => TokenType::Instruction(InstructionType::Atan2),
                        "ucmp_eq"          => TokenType::Instruction(InstructionType::UCmp(Comparison::Eq)),
                        "ucmp_ne"          => TokenType::Instruction(InstructionType::UCmp(Comparison::Neq)),
                        "ucmp_lt"          => TokenType::Instruction(InstructionType::UCmp(Comparison::Lt)),
                        "ucmp_lte"         => TokenType::Instruction(InstructionType::UCmp(Comparison::Lte)),
                        "and"              => TokenType::Instruction(InstructionType::And),
                        "andn"             => TokenType::Instruction(InstructionType::AndN),
                        "or"               => TokenType::Instruction(InstructionType::Or),
                        "xor"              => TokenType::Instruction(InstructionType::Xor),
                        "fma"              => TokenType::Instruction(InstructionType::Fma),
                        "lerp"             => TokenType::Instruction(InstructionType::Lerp),
                        "norm"             => TokenType::Instruction(InstructionType::Norm),
                        "mag"              => TokenType::Instruction(InstructionType::Mag),
                        "cross"            => TokenType::Instruction(InstructionType::Cross),
                        _                  => TokenType::Name,
                    }
                };
                tokens.push(Token {
                    t,
                    line: token_line,
                    column: token_column,
                    value: if t == TokenType::Name { Some(value_string.clone()) } else { None },
                });
            },

            '0' => {
                todo!()
            },

            '1' ..= '9' => {
                todo!()
            },

            '/' => {
                match chars.next() {
                    Some(c) => match c {
                        '*' => {
                            chars.next();
                            while let (Some(c1), Some(c2)) = (chars.next(), chars.peek()) {
                                match (c1, c2) {
                                    ('*', '/') => break,
                                    _ => {}
                                }
                            }
                            tokens.push(Token {
                                t: TokenType::Comment,
                                line: token_line,
                                column: token_column,
                                value: None,
                            });
                        },
                        '/' => {
                            chars.next();
                            while let Some(c) = chars.next() {
                                match c {
                                    '\n' => break,
                                    _ => {}
                                }
                            }
                            tokens.push(Token {
                                t: TokenType::Comment,
                                line: token_line,
                                column: token_column,
                                value: None,
                            });
                        },
                        _ => {
                            errors.push(SourceError {
                                message: format!("Unexpected character '{}'", c),
                                line: token_line,
                                column: token_column,
                            });
                        }
                    },
                    None => {
                        errors.push(SourceError {
                            message: format!("Unexpected character '{}'", c),
                            line: token_line,
                            column: token_column,
                        });
                    }
                }
            },

            _ => {
                errors.push(SourceError {
                    message: format!("Unexpected character '{}'", c),
                    line: token_line,
                    column: token_column,
                });
            }
        }
    }
    if errors.len() != 0 {
        Err(errors)
    } else {
        Ok(tokens)
    }
}
