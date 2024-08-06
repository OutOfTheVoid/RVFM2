use super::lexer::*;
use super::src_error::*;

trait WriteInstructionBytes {
    fn write(&self, bytes: &mut Vec<u8>, entry_type: EntryType);
}

const REGTYPE_LOCAL    : u8 = 0x00;
const REGTYPE_INPUT    : u8 = 0x01;
const REGTYPE_OUTPUT   : u8 = 0x02;
const REGTYPE_CONSTANT : u8 = 0x03;

impl WriteInstructionBytes for RegisterName {
    fn write(&self, bytes: &mut Vec<u8>, entry_type: EntryType) {
        let (regnum, regtype) = match (entry_type, self) {
            (_, RegisterName::LocalS(x)   ) => (*x, REGTYPE_LOCAL   ),
            (_, RegisterName::LocalV(x)   ) => (*x, REGTYPE_LOCAL   ),
            (_, RegisterName::ConstantS(x)) => (*x, REGTYPE_CONSTANT),
            (_, RegisterName::ConstantV(x)) => (*x, REGTYPE_CONSTANT),

            (_, RegisterName::InputS(x)   ) => (*x + 0x10, REGTYPE_INPUT   ),
            (_, RegisterName::InputV(x)   ) => (*x + 0x10, REGTYPE_INPUT   ),

            (_, RegisterName::OutputS(x)  ) => (*x + 0x10, REGTYPE_OUTPUT  ),
            (_, RegisterName::OutputV(x)  ) => (*x + 0x10, REGTYPE_OUTPUT  ),

            (EntryType::Vertex,   RegisterName::BuiltinS(ScalarBuiltin::VertexId       )) => (0x00, REGTYPE_INPUT ),
            (EntryType::Vertex,   RegisterName::BuiltinS(ScalarBuiltin::ProvokingVertex)) => (0x01, REGTYPE_INPUT ),

            (EntryType::Vertex,   RegisterName::BuiltinS(ScalarBuiltin::Discard        )) => (0x00, REGTYPE_OUTPUT),
            (EntryType::Vertex,   RegisterName::BuiltinV(VectorBuiltin::VertexPosition )) => (0x00, REGTYPE_OUTPUT),

            (EntryType::Fragment, RegisterName::BuiltinV(VectorBuiltin::VertexPosition )) => (0x00, REGTYPE_INPUT ),
            (EntryType::Fragment, RegisterName::BuiltinV(VectorBuiltin::Barycentric    )) => (0x01, REGTYPE_INPUT ),
            (EntryType::Fragment, RegisterName::BuiltinV(VectorBuiltin::Linear         )) => (0x02, REGTYPE_INPUT ),
            (EntryType::Fragment, RegisterName::BuiltinV(VectorBuiltin::VertexIds      )) => (0x03, REGTYPE_INPUT ),

            (EntryType::Fragment, RegisterName::BuiltinS(ScalarBuiltin::Discard        )) => (0x00, REGTYPE_OUTPUT),
            (EntryType::Fragment, RegisterName::BuiltinS(ScalarBuiltin::Depth          )) => (0x01, REGTYPE_OUTPUT),

            _ => panic!("Attempt to write illegal register address"),
        };
        bytes.push(regtype);
        bytes.push(regnum);
    }
}

const INSTRUCTION_VECTOR_PUSH                          : u8 = 0x00;
const INSTRUCTION_SCALAR_PUSH                          : u8 = 0x01;
const INSTRUCTION_VECTOR_POP                           : u8 = 0x02;
const INSTRUCTION_SCALAR_POP                           : u8 = 0x03;
const INSTRUCTION_VECTOR_COPY                          : u8 = 0x04;
const INSTRUCTION_SCALAR_COPY                          : u8 = 0x05;
const INSTRUCTION_VECTOR_COMPONENT_TO_SCALAR_COPY      : u8 = 0x06;
const INSTRUCTION_SCALAR_TO_VECTOR_COMPONENT_COPY      : u8 = 0x07;
const INSTRUCTION_COND_VECTOR_COPY                     : u8 = 0x08;
const INSTRUCTION_COND_SCALAR_COPY                     : u8 = 0x09;
const INSTRUCTION_COND_VECTOR_COMPONENT_TO_SCALAR_COPY : u8 = 0x0A;
const INSTRUCTION_COND_SCALAR_TO_VECTOR_COMPONENT_COPY : u8 = 0x0B;
const INSTRUCTION_COMPARE_SCALAR_F32                   : u8 = 0x0C;
const INSTRUCTION_COMPARE_VECTOR_F32                   : u8 = 0x0D;
const INSTRUCTION_COMPARE_SCALAR_I32                   : u8 = 0x0E;
const INSTRUCTION_COMPARE_VECTOR_I32                   : u8 = 0x0F;
const INSTRUCTION_COMPARE_SCALAR_U32                   : u8 = 0x10;
const INSTRUCTION_COMPARE_VECTOR_U32                   : u8 = 0x11;
const INSTRUCTION_MATRIX_MULTIPLY_M44_V4               : u8 = 0x12;

pub fn write_push(bytes: &mut Vec<u8>, src: RegisterName, src_token: &Token, entry_type: EntryType) -> Result<(), SourceError> {
    bytes.push(if src.is_vector() { INSTRUCTION_VECTOR_PUSH } else { INSTRUCTION_SCALAR_PUSH });
    src.write(bytes, entry_type);
    Ok(())
}

pub fn write_pop(bytes: &mut Vec<u8>, dst: RegisterName, dst_token: &Token, entry_type: EntryType) -> Result<(), SourceError> {
    bytes.push(if dst.is_vector() { INSTRUCTION_VECTOR_POP } else { INSTRUCTION_SCALAR_POP });
    dst.write(bytes, entry_type);
    Ok(())
}

enum Component {
    X,
    Y,
    Z,
    W
}

impl Component {
    fn from_token(t: &Token) -> Result<Self, SourceError> {
        Ok(match t.value.as_ref().map(|s| s.as_str()).unwrap_or("") {
            "x" | "r" => Self::X,
            "y" | "g" => Self::Y,
            "z" | "b" => Self::Z,
            "w" | "a" => Self::W,
            _ => {
                Err(SourceError {
                    message: format!("Expected component name"),
                    line: t.line,
                    column: t.column,
                })?
            }
        })
    }
}

impl WriteInstructionBytes for Component {
    fn write(&self, bytes: &mut Vec<u8>, entry_type: EntryType) {
        bytes.push(match self {
            Self::X => 0x00,
            Self::Y => 0x01,
            Self::Z => 0x02,
            Self::W => 0x03,
        })
    }
}

pub fn write_mov(bytes: &mut Vec<u8>, mov_token: &Token, dst: RegisterName, dst_component: Option<&Token>, src: RegisterName, src_component: Option<&Token>, entry_type: EntryType) -> Result<(), SourceError> {
    Ok(match (dst.is_vector(), dst_component, src.is_vector(), src_component) {
        (true,  None,            true,  None           ) => {
            bytes.push(INSTRUCTION_VECTOR_COPY);
            dst.write(bytes, entry_type);
            src.write(bytes, entry_type);
        },

        (false, None,            false, None           ) => {
            bytes.push(INSTRUCTION_SCALAR_COPY);
            dst.write(bytes, entry_type);
            src.write(bytes, entry_type);
        },

        (false, None,            true,  Some(component)) => {
            let component = Component::from_token(component)?;
            bytes.push(INSTRUCTION_VECTOR_COMPONENT_TO_SCALAR_COPY);
            dst.write(bytes, entry_type);
            src.write(bytes, entry_type);
            component.write(bytes, entry_type);
        },

        (true,  Some(component), false, None           ) => {
            let component = Component::from_token(component)?;
            bytes.push(INSTRUCTION_SCALAR_TO_VECTOR_COMPONENT_COPY);
            dst.write(bytes, entry_type);
            component.write(bytes, entry_type);
            src.write(bytes, entry_type);
        },

        (false, None,            true,  None           ) |
        (true,  None,            false, None           ) => {
            Err(SourceError {
                message: format!("Type mismatch between dst and src"),
                line: mov_token.line,
                column: mov_token.column
            })?
        },

        (false, Some(component), _,     _              ) |
        (_,     _,               false, Some(component)) => {
            Err(SourceError {
                message: format!("Scalar registers don't have components"),
                line: component.line,
                column: component.column
            })?
        },

        (true,  Some(component), true,  _              ) |
        (true,  _,               true,  Some(component)) => {
            Err(SourceError {
                message: format!("No mov instruction exists for two vector components"),
                line: component.line,
                column: component.column
            })?
        }
    })
}


pub fn write_cmov(bytes: &mut Vec<u8>, mov_token: &Token, cond: RegisterName, dst: RegisterName, dst_component: Option<&Token>, src: RegisterName, src_component: Option<&Token>, entry_type: EntryType) -> Result<(), SourceError> {
    println!("write_cmov(cond: {:?}, dst: {:?}, dst_component: {:?}, src: {:?}, src_component: {:?}, entry_type: {:?}", cond, dst, dst_component, src, src_component, entry_type);
    Ok(match (dst.is_vector(), dst_component, src.is_vector(), src_component) {
        (true,  None,            true,  None           ) => {
            bytes.push(INSTRUCTION_COND_VECTOR_COPY);
            cond.write(bytes, entry_type);
            dst.write(bytes, entry_type);
            src.write(bytes, entry_type);
        },

        (false, None,            false, None           ) => {
            bytes.push(INSTRUCTION_COND_SCALAR_COPY);
            cond.write(bytes, entry_type);
            dst.write(bytes, entry_type);
            src.write(bytes, entry_type);
        },

        (false, None,            true,  Some(component)) => {
            let component = Component::from_token(component)?;
            bytes.push(INSTRUCTION_COND_VECTOR_COMPONENT_TO_SCALAR_COPY);
            cond.write(bytes, entry_type);
            dst.write(bytes, entry_type);
            src.write(bytes, entry_type);
            component.write(bytes, entry_type);
        },

        (true,  Some(component), false, None           ) => {
            println!("writing INSTRUCTION_COND_SCALAR_TO_VECTOR_COMPONENT_COPY");
            let component = Component::from_token(component)?;
            bytes.push(INSTRUCTION_COND_SCALAR_TO_VECTOR_COMPONENT_COPY);
            cond.write(bytes, entry_type);
            dst.write(bytes, entry_type);
            component.write(bytes, entry_type);
            src.write(bytes, entry_type);
        },

        (false, None,            true,  None           ) |
        (true,  None,            false, None           ) => {
            Err(SourceError {
                message: format!("Type mismatch between dst and src"),
                line: mov_token.line,
                column: mov_token.column
            })?
        },

        (false, Some(component), _,     _              ) |
        (_,     _,               false, Some(component)) => {
            Err(SourceError {
                message: format!("Scalar registers don't have components"),
                line: component.line,
                column: component.column
            })?
        },

        (true,  Some(component), true,  _              ) |
        (true,  _,               true,  Some(component)) => {
            Err(SourceError {
                message: format!("No cmov instruction exists for two vector components"),
                line: component.line,
                column: component.column
            })?
        }
    })
}

impl Comparison {
    fn write(&self, bytes: &mut Vec<u8>) {
        let byte = match self {
            Comparison::Eq  => 0x00,
            Comparison::Neq => 0x01,
            Comparison::Lt  => 0x02,
            Comparison::Lte => 0x03,
        };
        bytes.push(byte);
    }
}

pub fn write_fcmp(bytes: &mut Vec<u8>, cmp_token: &Token, dst: RegisterName, a: RegisterName, b: RegisterName, comparison: Comparison, entry_type: EntryType) -> Result<(), SourceError> {
    let opcode = match (a.is_vector(), b.is_vector(), dst.is_vector()) {
        (true, true, true)    => INSTRUCTION_COMPARE_VECTOR_F32,
        (false, false, false) => INSTRUCTION_COMPARE_SCALAR_F32,
        _ => {
            let bool_to_type_name = |x: bool| if x { "vector" } else { "scalar" };
            Err(SourceError {
                message: format!(
                    "Invalid operand types - dst: {}, a: {}, b: {}",
                    bool_to_type_name(dst.is_vector()),
                    bool_to_type_name(a.is_vector()),
                    bool_to_type_name(b.is_vector())
                ),
                line: cmp_token.line,
                column: cmp_token.column,
            })?
        }
    };
    bytes.push(opcode);
    dst.write(bytes, entry_type);
    a.write(bytes, entry_type);
    b.write(bytes, entry_type);
    comparison.write(bytes);
    Ok(())
}

pub fn write_cmp(bytes: &mut Vec<u8>, cmp_token: &Token, dst: RegisterName, a: RegisterName, b: RegisterName, comparison: Comparison, entry_type: EntryType) -> Result<(), SourceError> {
    let opcode = match (a.is_vector(), b.is_vector(), dst.is_vector()) {
        (true, true, true)    => INSTRUCTION_COMPARE_VECTOR_I32,
        (false, false, false) => INSTRUCTION_COMPARE_SCALAR_I32,
        _ => {
            let bool_to_type_name = |x: bool| if x { "vector" } else { "scalar" };
            Err(SourceError {
                message: format!(
                    "Invalid operand types - dst: {}, a: {}, b: {}",
                    bool_to_type_name(dst.is_vector()),
                    bool_to_type_name(a.is_vector()),
                    bool_to_type_name(b.is_vector())
                ),
                line: cmp_token.line,
                column: cmp_token.column,
            })?
        }
    };
    bytes.push(opcode);
    dst.write(bytes, entry_type);
    a.write(bytes, entry_type);
    b.write(bytes, entry_type);
    comparison.write(bytes);
    Ok(())
}

pub fn write_ucmp(bytes: &mut Vec<u8>, cmp_token: &Token, dst: RegisterName, a: RegisterName, b: RegisterName, comparison: Comparison, entry_type: EntryType) -> Result<(), SourceError> {
    let opcode = match (a.is_vector(), b.is_vector(), dst.is_vector()) {
        (true, true, true)    => INSTRUCTION_COMPARE_VECTOR_U32,
        (false, false, false) => INSTRUCTION_COMPARE_SCALAR_U32,
        _ => {
            let bool_to_type_name = |x: bool| if x { "vector" } else { "scalar" };
            Err(SourceError {
                message: format!(
                    "Invalid operand types - dst: {}, a: {}, b: {}",
                    bool_to_type_name(dst.is_vector()),
                    bool_to_type_name(a.is_vector()),
                    bool_to_type_name(b.is_vector())
                ),
                line: cmp_token.line,
                column: cmp_token.column,
            })?
        }
    };
    bytes.push(opcode);
    dst.write(bytes, entry_type);
    a.write(bytes, entry_type);
    b.write(bytes, entry_type);
    comparison.write(bytes);
    Ok(())
}

pub fn write_mul_m44_v4(bytes: &mut Vec<u8>, mul_token: &Token, dst: RegisterName, a0: RegisterName, a1: RegisterName, a2: RegisterName, a3: RegisterName, x: RegisterName, entry_type: EntryType) -> Result<(), SourceError> {
    match dst.is_vector() && a0.is_vector() && a1.is_vector() && a2.is_vector() && a3.is_vector() && x.is_vector() {
        true => {},
        false =>{
            let bool_to_type_name = |x: bool| if x { "vector" } else { "scalar" };
            Err(SourceError {
                message: format!(
                    "Invalid operand types - dst: {}, a0: {}, a1: {}, a2: {}, a3: {}, x: {}",
                    bool_to_type_name(dst.is_vector()),
                    bool_to_type_name(a0.is_vector()),
                    bool_to_type_name(a1.is_vector()),
                    bool_to_type_name(a2.is_vector()),
                    bool_to_type_name(a3.is_vector()),
                    bool_to_type_name(x.is_vector())
                ),
                line: mul_token.line,
                column: mul_token.column,
            })?
        }
    }
    bytes.push(INSTRUCTION_MATRIX_MULTIPLY_M44_V4);
    dst.write(bytes, entry_type);
    a0.write(bytes, entry_type);
    a1.write(bytes, entry_type);
    a2.write(bytes, entry_type);
    a3.write(bytes, entry_type);
    x.write(bytes, entry_type);
    Ok(())
}
