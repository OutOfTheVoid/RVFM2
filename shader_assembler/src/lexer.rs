use super::src_error::*;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum AssemblyMode {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum CommandType {
    SetMode(AssemblyMode),
    Entry,
    Alias,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OpDataType {
    F32,
    I32
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
    ConvertF32ToI32,
    ConvertF32ToU32,
    ConvertU32ToF32,
    ConvertI32ToF32,
    Neg(OpDataType),
    Sign(OpDataType),
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
    FCmp(Comparison),
    Add(OpDataType),
    Sub(OpDataType),
    Mul(OpDataType),
    Div(OpDataType),
    Mod(OpDataType),
    Atan2,
    UCmp(Comparison),
    And,
    AndN,
    Or,
    Xor,
    Fma(OpDataType),
    Lerp,
    Norm2,
    Norm3,
    Norm4,
    Mag2,
    Mag3,
    Mag4,
    Cross,
    MatrixMultiply4x4V4,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum VectorBuiltin {
    VertexPosition,
    Barycentric,
    Linear,
    VertexIds
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
    Colon,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub t: TokenType,
    pub line: usize,
    pub column: usize,
    pub value: Option<String>,
}

enum NumberLexState {
    Integer,
    Float,
    FloatDecimal,
    FloatExponent(bool),
    FloatExponentValue,
}

pub fn run_lexer(input: &str) -> Result<Vec<Token>, Vec<SourceError>>  {
    let mut line = 1;
    let mut column = 1;
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut chars = input.chars().peekable();
    let mut pending_negative = false;
    let mut pend_negative = false;
    while let Some(c) = chars.next() {
        let mut value_string = String::new();
        let token_line = line;
        let token_column = column;
        column += 1;
        pending_negative = match (pending_negative, pend_negative) {
            (true, true) => {
                errors.push(SourceError { message: format!("Unexpected minus sign after minus sign"),
                    line: token_line,
                    column: token_column
                });
                false
            },
            (true, false) => false,
            (false, true) => true,
            (false, false) => false,
        };
        pend_negative = false;
        match c {
            ' ' | '\t' | '\n' => {
                pend_negative = pending_negative;
                pending_negative = false;
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
            },

            '.' => {
                tokens.push(Token {
                    t: TokenType::Dot,
                    line: token_line,
                    column: token_column,
                    value: None,
                });
            },

            ':' => {
                tokens.push(Token {
                    t: TokenType::Colon,
                    line: token_line,
                    column: token_column,
                    value: None,
                });
            }

            'a' ..= 'z' | 'A' ..= 'Z' | '_' => {
                value_string.push(c);
                while let Some(&c) = chars.peek() {
                    match c {
                        'a' ..= 'z' |
                        'A' ..= 'Z' |
                        '_'         |
                        '0' ..= '9' |
                        '.'         => {
                            value_string.push(c);
                            chars.next();
                            column += 1;
                        },
                        _ => break,
                    }
                }
                if let Some(&c) = chars.peek() {
                    if c == '!' {
                        value_string.push(c);
                        chars.next();
                        column += 1;
                    }
                }
                let (register_string, component_string) = match value_string.find('.') {
                    Some(index) => (value_string[..index].to_lowercase(), Some(value_string[index..].to_lowercase())),
                    None => (value_string.to_lowercase(), None)
                };
                println!("value_string: {}", value_string);
                let component_tokens = match component_string {
                    Some(component_string) => match component_string.as_str() {
                        ".x" | ".y" | ".z" | ".w" | ".r" | ".g" | ".b" | ".a" => Some(Ok([
                            Token{t: TokenType::Dot, line, column: column + register_string.len(), value: None},
                            Token{t: TokenType::Name, line, column: column + register_string.len() + 1, value: Some(component_string[1..].to_string())}
                        ])),
                        _ => {
                            Some(Err::<[Token; 2], SourceError>(SourceError {
                                message: format!("Invalid register component access specifier: {}", component_string),
                                line,
                                column,
                            }))
                        }
                    },
                    None => None
                };
                let (t, accepts_component, clear_component) = if register_string.to_lowercase().starts_with("vloc_") {
                    if let Ok(index) = register_string[5..].parse::<u32>() {
                        (TokenType::Register(RegisterName::LocalV(index as u8)), true, false)
                    } else {
                        errors.push(SourceError {
                            message: format!("Expected index after vloc_"),
                            line: token_line,
                            column: token_column,
                        });
                        continue;
                    }
                } else if register_string.to_lowercase().starts_with("sloc_") {
                    if let Ok(index) = register_string[5..].parse::<u32>() {
                        (TokenType::Register(RegisterName::LocalS(index as u8)), false, false)
                    } else {
                        errors.push(SourceError {
                            message: format!("Expected index after sloc_ "),
                            line: token_line,
                            column: token_column,
                        });
                        continue;
                    }
                } else if register_string.to_lowercase().starts_with("vin_") {
                    if let Ok(index) = register_string[4..].parse::<u32>() {
                        (TokenType::Register(RegisterName::InputV(index as u8)), true, false)
                    } else {
                        errors.push(SourceError {
                            message: format!("Expected index after vin_ - token: {}", register_string),
                            line: token_line,
                            column: token_column,
                        });
                        continue;
                    }
                } else if register_string.to_lowercase().starts_with("sin_") {
                    if let Ok(index) = register_string[4..].parse::<u32>() {
                        (TokenType::Register(RegisterName::InputS(index as u8)), false, false)
                    } else {
                        errors.push(SourceError {
                            message: format!("Expected index after sin_ "),
                            line: token_line,
                            column: token_column,
                        });
                        continue;
                    }
                } else if register_string.to_lowercase().starts_with("vout_") {
                    if let Ok(index) = register_string[5..].parse::<u32>() {
                        (TokenType::Register(RegisterName::OutputV(index as u8)), true, false)
                    } else {
                        errors.push(SourceError {
                            message: format!("Expected index after vout_, found: {}", register_string),
                            line: token_line,
                            column: token_column,
                        });
                        continue;
                    }
                } else if register_string.to_lowercase().starts_with("sout_") {
                    if let Ok(index) = register_string[5..].parse::<u32>() {
                        (TokenType::Register(RegisterName::OutputS(index as u8)), false, false)
                    } else {
                        errors.push(SourceError {
                            message: format!("Expected index after sout_ "),
                            line: token_line,
                            column: token_column,
                        });
                        continue;
                    }
                } else if register_string.to_lowercase().starts_with("vconst_") {
                    if let Ok(index) = register_string[7..].parse::<u32>() {
                        (TokenType::Register(RegisterName::ConstantV(index as u8)), true, false)
                    } else {
                        errors.push(SourceError {
                            message: format!("Expected index after vconst_ "),
                            line: token_line,
                            column: token_column,
                        });
                        continue;
                    }
                } else if register_string.to_lowercase().starts_with("sconst_") {
                    if let Ok(index) = register_string[7..].parse::<u32>() {
                        (TokenType::Register(RegisterName::ConstantS(index as u8)), false, false)
                    } else {
                        errors.push(SourceError {
                            message: format!("Expected index after sconst_ "),
                            line: token_line,
                            column: token_column,
                        });
                        continue;
                    }
                } else {
                    match value_string.to_lowercase().as_str() {
                        "vertex_position"  => (TokenType::Register(RegisterName::BuiltinV(VectorBuiltin::VertexPosition)), false, false),
                        "barycentric"      => (TokenType::Register(RegisterName::BuiltinV(VectorBuiltin::Barycentric)), false, false),
                        "linear"           => (TokenType::Register(RegisterName::BuiltinV(VectorBuiltin::Linear)), false, false),
                        "vertex_ids"       => (TokenType::Register(RegisterName::BuiltinV(VectorBuiltin::VertexIds)), false, false),
                        "depth"            => (TokenType::Register(RegisterName::BuiltinS(ScalarBuiltin::Depth)), false, false),
                        "vertex_id"        => (TokenType::Register(RegisterName::BuiltinS(ScalarBuiltin::VertexId)), false, false),
                        "provoking_vertex" => (TokenType::Register(RegisterName::BuiltinS(ScalarBuiltin::ProvokingVertex)), false, false),
                        "discard"          => (TokenType::Register(RegisterName::BuiltinS(ScalarBuiltin::Discard)), false, false),

                        "vertex!"          => (TokenType::Command(CommandType::SetMode(AssemblyMode::Vertex)), false, false),
                        "fragment!"        => (TokenType::Command(CommandType::SetMode(AssemblyMode::Fragment)), false, false),
                        "compute!"         => (TokenType::Command(CommandType::SetMode(AssemblyMode::Compute)), false, false),
                        "entry!"           => (TokenType::Command(CommandType::Entry), false, false),
                        "alias!"           => (TokenType::Command(CommandType::Alias), false, false),
                        "texture!"         => (TokenType::Command(CommandType::TextureDecl), false, false),
                        "buffer!"          => (TokenType::Command(CommandType::BufferDecl), false, false),

                        "push"             => (TokenType::Instruction(InstructionType::Push), false, false),
                        "pop"              => (TokenType::Instruction(InstructionType::Pop), false, false),
                        "mov"              => (TokenType::Instruction(InstructionType::Mov), false, false),
                        "cmov"             => (TokenType::Instruction(InstructionType::CMov), false, false),
                        "read"             => (TokenType::Instruction(InstructionType::Read), false, false),
                        "write"            => (TokenType::Instruction(InstructionType::Write), false, false),
                        "cread"            => (TokenType::Instruction(InstructionType::CRead), false, false),
                        "cwrite"           => (TokenType::Instruction(InstructionType::CWrite), false, false),
                        //"load"             => (TokenType::Instruction(InstructionType::Load), false, false),
                        //"store"            => (TokenType::Instruction(InstructionType::Store), false, false),
                        "conv.i32.f32"     => (TokenType::Instruction(InstructionType::ConvertF32ToI32), false, true),
                        "conv.u32.f32"     => (TokenType::Instruction(InstructionType::ConvertF32ToU32), false, true),
                        "conv.f32.i32"     => (TokenType::Instruction(InstructionType::ConvertI32ToF32), false, true),
                        "conv.f32.u32"     => (TokenType::Instruction(InstructionType::ConvertU32ToF32), false, true),
                        "neg.i"            => (TokenType::Instruction(InstructionType::Neg(OpDataType::I32)), false, true),
                        "neg.f"            => (TokenType::Instruction(InstructionType::Neg(OpDataType::F32)), false, true),
                        "sign.f"           => (TokenType::Instruction(InstructionType::Sign(OpDataType::F32)), false, true),
                        "sign.i"           => (TokenType::Instruction(InstructionType::Sign(OpDataType::I32)), false, true),
                        "recip"            => (TokenType::Instruction(InstructionType::Recip), false, false),
                        "sin"              => (TokenType::Instruction(InstructionType::Sin), false, false),
                        "cos"              => (TokenType::Instruction(InstructionType::Cos), false, false),
                        "tan"              => (TokenType::Instruction(InstructionType::Tan), false, false),
                        "asin"             => (TokenType::Instruction(InstructionType::ASin), false, false),
                        "acos"             => (TokenType::Instruction(InstructionType::ACos), false, false),
                        "atan"             => (TokenType::Instruction(InstructionType::Atan), false, false),
                        "ln"               => (TokenType::Instruction(InstructionType::Ln), false, false),
                        "exp"              => (TokenType::Instruction(InstructionType::Exp), false, false),
                        "cmp.i.eq"         => (TokenType::Instruction(InstructionType::Cmp(Comparison::Eq)), false, true),
                        "cmp.i.neq"        => (TokenType::Instruction(InstructionType::Cmp(Comparison::Neq)), false, true),
                        "cmp.i.lt"         => (TokenType::Instruction(InstructionType::Cmp(Comparison::Lt)), false, true),
                        "cmp.i.lte"        => (TokenType::Instruction(InstructionType::Cmp(Comparison::Lte)), false, true),
                        "cmp.f.eq"         => (TokenType::Instruction(InstructionType::FCmp(Comparison::Eq)), false, true),
                        "cmp.f.neq"        => (TokenType::Instruction(InstructionType::FCmp(Comparison::Neq)), false, true),
                        "cmp.f.lt"         => (TokenType::Instruction(InstructionType::FCmp(Comparison::Lt)), false, true),
                        "cmp.f.lte"        => (TokenType::Instruction(InstructionType::FCmp(Comparison::Lte)), false, true),
                        "cmp.u.eq"         => (TokenType::Instruction(InstructionType::UCmp(Comparison::Eq)), false, true),
                        "cmp.u.neq"        => (TokenType::Instruction(InstructionType::UCmp(Comparison::Neq)), false, true),
                        "cmp.u.lt"         => (TokenType::Instruction(InstructionType::UCmp(Comparison::Lt)), false, true),
                        "cmp.u.lte"        => (TokenType::Instruction(InstructionType::UCmp(Comparison::Lte)), false, true),
                        "add.f"            => (TokenType::Instruction(InstructionType::Add(OpDataType::F32)), false, true),
                        "sub.f"            => (TokenType::Instruction(InstructionType::Sub(OpDataType::F32)), false, true),
                        "mul.f"            => (TokenType::Instruction(InstructionType::Mul(OpDataType::F32)), false, true),
                        "div.f"            => (TokenType::Instruction(InstructionType::Div(OpDataType::F32)), false, true),
                        "mod.f"            => (TokenType::Instruction(InstructionType::Mod(OpDataType::F32)), false, true),
                        "add.i"            => (TokenType::Instruction(InstructionType::Add(OpDataType::I32)), false, true),
                        "sub.i"            => (TokenType::Instruction(InstructionType::Sub(OpDataType::I32)), false, true),
                        "mul.i"            => (TokenType::Instruction(InstructionType::Mul(OpDataType::I32)), false, true),
                        "div.i"            => (TokenType::Instruction(InstructionType::Div(OpDataType::I32)), false, true),
                        "mod.i"            => (TokenType::Instruction(InstructionType::Mod(OpDataType::I32)), false, true),
                        "atan2"            => (TokenType::Instruction(InstructionType::Atan2), false, false),
                        "and"              => (TokenType::Instruction(InstructionType::And), false, false),
                        "andn"             => (TokenType::Instruction(InstructionType::AndN), false, false),
                        "or"               => (TokenType::Instruction(InstructionType::Or), false, false),
                        "xor"              => (TokenType::Instruction(InstructionType::Xor), false, false),
                        "ffma"             => (TokenType::Instruction(InstructionType::Fma(OpDataType::F32)), false, false),
                        "ifma"             => (TokenType::Instruction(InstructionType::Fma(OpDataType::I32)), false, false),
                        "lerp"             => (TokenType::Instruction(InstructionType::Lerp), false, false),
                        "norm.2"           => (TokenType::Instruction(InstructionType::Norm2), false, true),
                        "norm.3"           => (TokenType::Instruction(InstructionType::Norm3), false, true),
                        "norm.4"           => (TokenType::Instruction(InstructionType::Norm4), false, true),
                        "mag.v2"           => (TokenType::Instruction(InstructionType::Mag2), false, true),
                        "mag.v3"           => (TokenType::Instruction(InstructionType::Mag3), false, true),
                        "mag.v4"           => (TokenType::Instruction(InstructionType::Mag4), false, true),
                        "cross"            => (TokenType::Instruction(InstructionType::Cross), false, false),
                        "mul.m44.v4"       => (TokenType::Instruction(InstructionType::MatrixMultiply4x4V4), false, true),
                        _                  => (TokenType::Name, true, false),
                    }
                };
                println!("LEXER PUSHING TOKEN: {:?}", t);
                tokens.push(Token {
                    t,
                    line: token_line,
                    column: token_column,
                    value: if t == TokenType::Name { Some(register_string.clone()) } else { None },
                });
                let component_tokens = if clear_component { None } else { component_tokens };
                match (accepts_component, component_tokens) {
                    (_, None) => {},
                    (false, Some(accesor)) => {
                        errors.push(SourceError { message: format!("Unexpected channel accessor: {:?}", accesor), line, column });
                    },
                    (true, Some(Ok([dot, component]))) => {
                        tokens.push(dot);
                        tokens.push(component);
                    },
                    (true, Some(Err(error))) => {
                        errors.push(error);
                    }
                }
            },

            '-' => {
                pend_negative = true;
            },

            '0' => {
                let negative = pending_negative;
                pending_negative = false;
                if let Some(format_char) = chars.peek() {
                    match *format_char {
                        'x' | 'X' | 'h' | 'H' => {
                            chars.next();
                            column += 1;
                            let mut value_string = String::new();
                            while let Some(c) = chars.peek() {
                                let is_digit = match c {
                                    '0' ..= '9' => true,
                                    'a' ..= 'f' => true,
                                    'A' ..= 'F' => true,
                                    _ => false
                                };
                                if is_digit {
                                    value_string.push(*c);
                                    column += 1;
                                } else {
                                    if value_string.len() == 0 {
                                        errors.push(SourceError {
                                            message: format!("Expected value digits after hexadecimal start sequence"),
                                            line: token_line,
                                            column: token_column,
                                        });
                                        break;
                                    } else {
                                        tokens.push(Token {
                                            t: TokenType::Number,
                                            line: token_line,
                                            column: token_column,
                                            value: Some(if negative { "-".to_string() + &value_string } else { value_string.clone() })
                                        });
                                        break;
                                    }
                                }
                            }
                        },
                        'b' | 'B' => {
                            chars.next();
                            column += 1;
                            let mut value_string = String::new();
                            while let Some(c) = chars.peek() {
                                let is_digit = match c {
                                    '0' ..= '1' => true,
                                    _ => false
                                };
                                if is_digit {
                                    value_string.push(*c);
                                    column += 1;
                                } else {
                                    if value_string.len() == 0 {
                                        errors.push(SourceError {
                                            message: format!("Expected value digits after binary start sequence"),
                                            line: token_line,
                                            column: token_column,
                                        });
                                        break;
                                    } else {
                                        tokens.push(Token {
                                            t: TokenType::Number,
                                            line: token_line,
                                            column: token_column,
                                            value: Some(if negative { "-".to_string() + &value_string } else { value_string.clone() })
                                        });
                                        break;
                                    }
                                }
                            }
                        },
                        'o' | 'O' => {
                            chars.next();
                            column += 1;
                            let mut value_string = String::new();
                            while let Some(c) = chars.peek() {
                                let is_digit = match c {
                                    '0' ..= '7' => true,
                                    _ => false
                                };
                                if is_digit {
                                    value_string.push(*c);
                                    column += 1;
                                } else {
                                    if value_string.len() == 0 {
                                        errors.push(SourceError {
                                            message: format!("Expected value digits after octal start sequence"),
                                            line: token_line,
                                            column: token_column,
                                        });
                                        break;
                                    } else {
                                        tokens.push(Token {
                                            t: TokenType::Number,
                                            line: token_line,
                                            column: token_column,
                                            value: Some(if negative { "-".to_string() + &value_string } else { value_string.clone() })
                                        });
                                        break;
                                    }
                                }
                            }
                        },
                        _ => {
                            let mut state = NumberLexState::Integer;
                            let mut value_string = String::new();
                            pending_negative = false;
                            let mut error = None;
                            while let Some(&c) = chars.peek() {
                                let new_state = match (state, c) {
                                    (NumberLexState::Integer, '0' ..= '9') => {
                                        value_string.push(c);
                                        column += 1;
                                        chars.next();
                                        NumberLexState::Integer
                                    },
                                    (NumberLexState::Integer, '.') => {
                                        value_string.push(c);
                                        column += 1;
                                        chars.next();
                                        NumberLexState::FloatDecimal
                                    },
                                    (NumberLexState::Integer, _) => {
                                        break;
                                    },
                                    (NumberLexState::FloatDecimal, '0' ..= '9') => {
                                        value_string.push(c);
                                        column += 1;
                                        chars.next();
                                        NumberLexState::Float
                                    },
                                    (NumberLexState::FloatDecimal, _) => {
                                        error = Some(SourceError {
                                            message: format!("Expected a numeric digit after dot in float immediate"),
                                            line,
                                            column,
                                        });
                                        break;
                                    },
                                    (NumberLexState::Float, '0' ..= '9') => {
                                        value_string.push(c);
                                        column += 1;
                                        chars.next();
                                        NumberLexState::Float
                                    },
                                    (NumberLexState::Float, 'e') => {
                                        value_string.push(c);
                                        column += 1;
                                        chars.next();
                                        NumberLexState::FloatExponent(false)
                                    },
                                    (NumberLexState::Float, _) => {
                                        break;
                                    },
                                    (NumberLexState::FloatExponent(false), '-') => {
                                        value_string.push(c);
                                        column += 1;
                                        chars.next();
                                        NumberLexState::FloatExponent(true)
                                    },
                                    (NumberLexState::FloatExponent(_), '0' ..= '9') => {
                                        value_string.push(c);
                                        column += 1;
                                        chars.next();
                                        NumberLexState::FloatExponentValue
                                    },
                                    (NumberLexState::FloatExponent(_), _) => {
                                        error = Some(SourceError {
                                            message: format!("Expected a numeric digit e in float immediate"),
                                            line,
                                            column,
                                        });
                                        break;
                                    },
                                    (NumberLexState::FloatExponentValue, '0' ..= '9') => {
                                        value_string.push(c);
                                        column += 1;
                                        chars.next();
                                        NumberLexState::FloatExponentValue
                                    },
                                    (NumberLexState::FloatExponentValue, _) => {
                                        break;
                                    }
                                };
                                state = new_state;
                            }
                            if let Some(error) = error {
                                errors.push(error);
                            } else {
                                tokens.push(Token { 
                                    t : TokenType::Number,
                                    line: token_line,
                                    column: token_column,
                                    value: Some(value_string)
                                });
                            }
                        }
                    }
                }
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
