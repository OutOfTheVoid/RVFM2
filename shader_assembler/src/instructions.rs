use super::lexer::*;
use super::src_error::*;

trait WriteInstructionBytes {
    fn write(&self, bytes: &mut Vec<u8>, assembly_mode: AssemblyMode);
}

const REGTYPE_LOCAL    : u8 = 0x00;
const REGTYPE_INPUT    : u8 = 0x01;
const REGTYPE_OUTPUT   : u8 = 0x02;
const REGTYPE_CONSTANT : u8 = 0x03;

impl WriteInstructionBytes for RegisterName {
    fn write(&self, bytes: &mut Vec<u8>, assembly_mode: AssemblyMode) {
        let (regnum, regtype) = match (assembly_mode, self) {
            (_, RegisterName::LocalS(x)   ) => (*x, REGTYPE_LOCAL   ),
            (_, RegisterName::LocalV(x)   ) => (*x, REGTYPE_LOCAL   ),
            (_, RegisterName::ConstantS(x)) => (*x, REGTYPE_CONSTANT),
            (_, RegisterName::ConstantV(x)) => (*x, REGTYPE_CONSTANT),

            (_, RegisterName::InputS(x)   ) => (*x + 0x10, REGTYPE_INPUT   ),
            (_, RegisterName::InputV(x)   ) => (*x + 0x10, REGTYPE_INPUT   ),

            (_, RegisterName::OutputS(x)  ) => (*x + 0x10, REGTYPE_OUTPUT  ),
            (_, RegisterName::OutputV(x)  ) => (*x + 0x10, REGTYPE_OUTPUT  ),

            (AssemblyMode::Vertex,   RegisterName::BuiltinS(ScalarBuiltin::VertexId       )) => (0x00, REGTYPE_INPUT ),
            (AssemblyMode::Vertex,   RegisterName::BuiltinS(ScalarBuiltin::ProvokingVertex)) => (0x01, REGTYPE_INPUT ),

            (AssemblyMode::Vertex,   RegisterName::BuiltinS(ScalarBuiltin::Discard        )) => (0x00, REGTYPE_OUTPUT),
            (AssemblyMode::Vertex,   RegisterName::BuiltinV(VectorBuiltin::VertexPosition )) => (0x00, REGTYPE_OUTPUT),

            (AssemblyMode::Fragment, RegisterName::BuiltinV(VectorBuiltin::VertexPosition )) => (0x00, REGTYPE_INPUT ),
            (AssemblyMode::Fragment, RegisterName::BuiltinV(VectorBuiltin::Barycentric    )) => (0x01, REGTYPE_INPUT ),
            (AssemblyMode::Fragment, RegisterName::BuiltinV(VectorBuiltin::Linear         )) => (0x02, REGTYPE_INPUT ),
            (AssemblyMode::Fragment, RegisterName::BuiltinV(VectorBuiltin::VertexIds      )) => (0x03, REGTYPE_INPUT ),

            (AssemblyMode::Fragment, RegisterName::BuiltinS(ScalarBuiltin::Discard        )) => (0x00, REGTYPE_OUTPUT),
            (AssemblyMode::Fragment, RegisterName::BuiltinS(ScalarBuiltin::Depth          )) => (0x01, REGTYPE_OUTPUT),

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

const INSTRUCTION_SCALAR_ADD_F32                       : u8 = 0x13;
const INSTRUCTION_SCALAR_SUB_F32                       : u8 = 0x14;
const INSTRUCTION_SCALAR_MUL_F32                       : u8 = 0x15;
const INSTRUCTION_SCALAR_DIV_F32                       : u8 = 0x16;
const INSTRUCTION_SCALAR_MOD_F32                       : u8 = 0x17;
const INSTRUCTION_SCALAR_ADD_I32                       : u8 = 0x18;
const INSTRUCTION_SCALAR_SUB_I32                       : u8 = 0x19;
const INSTRUCTION_SCALAR_MUL_I32                       : u8 = 0x1A;
const INSTRUCTION_SCALAR_DIV_I32                       : u8 = 0x1B;
const INSTRUCTION_SCALAR_MOD_I32                       : u8 = 0x1C;


const INSTRUCTION_VECTOR_CW_ADD_F32                    : u8 = 0x1D;
const INSTRUCTION_VECTOR_CW_SUB_F32                    : u8 = 0x1E;
const INSTRUCTION_VECTOR_CW_MUL_F32                    : u8 = 0x1F;
const INSTRUCTION_VECTOR_CW_DIV_F32                    : u8 = 0x20;
const INSTRUCTION_VECTOR_CW_MOD_F32                    : u8 = 0x21;
const INSTRUCTION_VECTOR_CW_ADD_I32                    : u8 = 0x22;
const INSTRUCTION_VECTOR_CW_SUB_I32                    : u8 = 0x23;
const INSTRUCTION_VECTOR_CW_MUL_I32                    : u8 = 0x24;
const INSTRUCTION_VECTOR_CW_DIV_I32                    : u8 = 0x25;
const INSTRUCTION_VECTOR_CW_MOD_I32                    : u8 = 0x26;

const INSTRUCTION_SCALAR_CONV_F32_TO_I32               : u8 = 0x27;
const INSTRUCTION_SCALAR_CONV_F32_TO_U32               : u8 = 0x28;
const INSTRUCTION_SCALAR_CONV_U32_TO_F32               : u8 = 0x29;
const INSTRUCTION_SCALAR_CONV_I32_TO_F32               : u8 = 0x2A;
const INSTRUCTION_SCALAR_NEG_F32                       : u8 = 0x2B;
const INSTRUCTION_SCALAR_NEG_I32                       : u8 = 0x2C;
const INSTRUCTION_SCALAR_SIGN_F32                      : u8 = 0x2D;
const INSTRUCTION_SCALAR_SIGN_I32                      : u8 = 0x2E;
const INSTRUCTION_SCALAR_RECIP                         : u8 = 0x2F;
const INSTRUCTION_SCALAR_SIN                           : u8 = 0x30;
const INSTRUCTION_SCALAR_COS                           : u8 = 0x31;
const INSTRUCTION_SCALAR_TAN                           : u8 = 0x32;
const INSTRUCTION_SCALAR_ASIN                          : u8 = 0x33;
const INSTRUCTION_SCALAR_ACOS                          : u8 = 0x34;
const INSTRUCTION_SCALAR_ATAN                          : u8 = 0x35;
const INSTRUCTION_SCALAR_LN                            : u8 = 0x36;
const INSTRUCTION_SCALAR_EXP                           : u8 = 0x37;

const INSTRUCTION_VECTOR_CW_CONV_F32_TO_I32               : u8 = 0x38;
const INSTRUCTION_VECTOR_CW_CONV_F32_TO_U32               : u8 = 0x39;
const INSTRUCTION_VECTOR_CW_CONV_U32_TO_F32               : u8 = 0x3A;
const INSTRUCTION_VECTOR_CW_CONV_I32_TO_F32               : u8 = 0x3B;
const INSTRUCTION_VECTOR_CW_NEG_F32                       : u8 = 0x3C;
const INSTRUCTION_VECTOR_CW_NEG_I32                       : u8 = 0x3D;
const INSTRUCTION_VECTOR_CW_SIGN_F32                      : u8 = 0x3E;
const INSTRUCTION_VECTOR_CW_SIGN_I32                      : u8 = 0x3F;
const INSTRUCTION_VECTOR_CW_RECIP                         : u8 = 0x40;
const INSTRUCTION_VECTOR_CW_SIN                           : u8 = 0x41;
const INSTRUCTION_VECTOR_CW_COS                           : u8 = 0x42;
const INSTRUCTION_VECTOR_CW_TAN                           : u8 = 0x43;
const INSTRUCTION_VECTOR_CW_ASIN                          : u8 = 0x44;
const INSTRUCTION_VECTOR_CW_ACOS                          : u8 = 0x45;
const INSTRUCTION_VECTOR_CW_ATAN                          : u8 = 0x46;
const INSTRUCTION_VECTOR_CW_LN                            : u8 = 0x47;
const INSTRUCTION_VECTOR_CW_EXP                           : u8 = 0x48;

const INSTRUCTION_SCALAR_ATAN2                            : u8 = 0x49;
const INSTRUCTION_SCALAR_AND                              : u8 = 0x4A;
const INSTRUCTION_SCALAR_AND_NOT                          : u8 = 0x4B;
const INSTRUCTION_SCALAR_OR                               : u8 = 0x4C;
const INSTRUCTION_SCALAR_XOR                              : u8 = 0x4D;

const INSTRUCTION_VECTOR_CW_ATAN2                         : u8 = 0x4E;
const INSTRUCTION_VECTOR_CW_AND                           : u8 = 0x4F;
const INSTRUCTION_VECTOR_CW_AND_NOT                       : u8 = 0x50;
const INSTRUCTION_VECTOR_CW_OR                            : u8 = 0x51;
const INSTRUCTION_VECTOR_CW_XOR                           : u8 = 0x52;

pub fn write_push(bytes: &mut Vec<u8>, src: RegisterName, _src_token: &Token, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
    bytes.push(if src.is_vector() { INSTRUCTION_VECTOR_PUSH } else { INSTRUCTION_SCALAR_PUSH });
    src.write(bytes, assembly_mode);
    Ok(())
}

pub fn write_pop(bytes: &mut Vec<u8>, dst: RegisterName, _dst_token: &Token, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
    bytes.push(if dst.is_vector() { INSTRUCTION_VECTOR_POP } else { INSTRUCTION_SCALAR_POP });
    dst.write(bytes, assembly_mode);
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
    fn write(&self, bytes: &mut Vec<u8>, _assembly_mode: AssemblyMode) {
        bytes.push(match self {
            Self::X => 0x00,
            Self::Y => 0x01,
            Self::Z => 0x02,
            Self::W => 0x03,
        })
    }
}

fn bool_to_type_name(x: bool) -> &'static str {
    if x {
        "vector"
    } else {
        "scalar"
    }
}

pub fn write_mov(bytes: &mut Vec<u8>, mov_token: &Token, dst: RegisterName, dst_component: Option<&Token>, src: RegisterName, src_component: Option<&Token>, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
    Ok(match (dst.is_vector(), dst_component, src.is_vector(), src_component) {
        (true,  None,            true,  None           ) => {
            bytes.push(INSTRUCTION_VECTOR_COPY);
            dst.write(bytes, assembly_mode);
            src.write(bytes, assembly_mode);
        },

        (false, None,            false, None           ) => {
            bytes.push(INSTRUCTION_SCALAR_COPY);
            dst.write(bytes, assembly_mode);
            src.write(bytes, assembly_mode);
        },

        (false, None,            true,  Some(component)) => {
            let component = Component::from_token(component)?;
            bytes.push(INSTRUCTION_VECTOR_COMPONENT_TO_SCALAR_COPY);
            dst.write(bytes, assembly_mode);
            src.write(bytes, assembly_mode);
            component.write(bytes, assembly_mode);
        },

        (true,  Some(component), false, None           ) => {
            let component = Component::from_token(component)?;
            bytes.push(INSTRUCTION_SCALAR_TO_VECTOR_COMPONENT_COPY);
            dst.write(bytes, assembly_mode);
            component.write(bytes, assembly_mode);
            src.write(bytes, assembly_mode);
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


pub fn write_cmov(bytes: &mut Vec<u8>, mov_token: &Token, cond: RegisterName, dst: RegisterName, dst_component: Option<&Token>, src: RegisterName, src_component: Option<&Token>, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
    println!("write_cmov(cond: {:?}, dst: {:?}, dst_component: {:?}, src: {:?}, src_component: {:?}, assembly_mode: {:?}", cond, dst, dst_component, src, src_component, assembly_mode);
    Ok(match (dst.is_vector(), dst_component, src.is_vector(), src_component) {
        (true,  None,            true,  None           ) => {
            bytes.push(INSTRUCTION_COND_VECTOR_COPY);
            cond.write(bytes, assembly_mode);
            dst.write(bytes, assembly_mode);
            src.write(bytes, assembly_mode);
        },

        (false, None,            false, None           ) => {
            bytes.push(INSTRUCTION_COND_SCALAR_COPY);
            cond.write(bytes, assembly_mode);
            dst.write(bytes, assembly_mode);
            src.write(bytes, assembly_mode);
        },

        (false, None,            true,  Some(component)) => {
            let component = Component::from_token(component)?;
            bytes.push(INSTRUCTION_COND_VECTOR_COMPONENT_TO_SCALAR_COPY);
            cond.write(bytes, assembly_mode);
            dst.write(bytes, assembly_mode);
            src.write(bytes, assembly_mode);
            component.write(bytes, assembly_mode);
        },

        (true,  Some(component), false, None           ) => {
            println!("writing INSTRUCTION_COND_SCALAR_TO_VECTOR_COMPONENT_COPY");
            let component = Component::from_token(component)?;
            bytes.push(INSTRUCTION_COND_SCALAR_TO_VECTOR_COMPONENT_COPY);
            cond.write(bytes, assembly_mode);
            dst.write(bytes, assembly_mode);
            component.write(bytes, assembly_mode);
            src.write(bytes, assembly_mode);
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

pub fn write_fcmp(bytes: &mut Vec<u8>, cmp_token: &Token, dst: RegisterName, a: RegisterName, b: RegisterName, comparison: Comparison, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
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
    dst.write(bytes, assembly_mode);
    a.write(bytes, assembly_mode);
    b.write(bytes, assembly_mode);
    comparison.write(bytes);
    Ok(())
}

pub fn write_cmp(bytes: &mut Vec<u8>, cmp_token: &Token, dst: RegisterName, a: RegisterName, b: RegisterName, comparison: Comparison, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
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
    dst.write(bytes, assembly_mode);
    a.write(bytes, assembly_mode);
    b.write(bytes, assembly_mode);
    comparison.write(bytes);
    Ok(())
}

pub fn write_ucmp(bytes: &mut Vec<u8>, cmp_token: &Token, dst: RegisterName, a: RegisterName, b: RegisterName, comparison: Comparison, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
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
    dst.write(bytes, assembly_mode);
    a.write(bytes, assembly_mode);
    b.write(bytes, assembly_mode);
    comparison.write(bytes);
    Ok(())
}

pub fn write_mul_m44_v4(bytes: &mut Vec<u8>, mul_token: &Token, dst: RegisterName, a0: RegisterName, a1: RegisterName, a2: RegisterName, a3: RegisterName, x: RegisterName, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
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
    dst.write(bytes, assembly_mode);
    a0.write(bytes, assembly_mode);
    a1.write(bytes, assembly_mode);
    a2.write(bytes, assembly_mode);
    a3.write(bytes, assembly_mode);
    x.write(bytes, assembly_mode);
    Ok(())
}

pub fn write_scalar_binary_op(bytes: &mut Vec<u8>, op_token: &Token, op: InstructionType, dst: RegisterName, a: RegisterName, b: RegisterName, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
    let vector = match (dst.is_vector(), a.is_vector(), b.is_vector()) {
        (true, true, true) => true,
        (false, false, false) => false,
        _ => {
            Err(SourceError {
                message: format!(
                    "Invalid operand types - dst: {}, a: {}, b: {}",
                    bool_to_type_name(dst.is_vector()),
                    bool_to_type_name(a.is_vector()),
                    bool_to_type_name(b.is_vector())
                ),
                line: op_token.line,
                column: op_token.column,
            })?
        }
    };
    let opcode = match (op, vector) {
        (InstructionType::Add(OpDataType::F32), false ) => INSTRUCTION_SCALAR_ADD_F32,
        (InstructionType::Add(OpDataType::I32), false ) => INSTRUCTION_SCALAR_ADD_I32,
        (InstructionType::Add(OpDataType::F32), true  ) => INSTRUCTION_VECTOR_CW_ADD_F32,
        (InstructionType::Add(OpDataType::I32), true  ) => INSTRUCTION_VECTOR_CW_ADD_I32,

        (InstructionType::Sub(OpDataType::F32), false ) => INSTRUCTION_SCALAR_SUB_F32,
        (InstructionType::Sub(OpDataType::I32), false ) => INSTRUCTION_SCALAR_SUB_I32,
        (InstructionType::Sub(OpDataType::F32), true  ) => INSTRUCTION_VECTOR_CW_SUB_F32,
        (InstructionType::Sub(OpDataType::I32), true  ) => INSTRUCTION_VECTOR_CW_SUB_I32,

        (InstructionType::Mul(OpDataType::F32), false ) => INSTRUCTION_SCALAR_MUL_F32,
        (InstructionType::Mul(OpDataType::I32), false ) => INSTRUCTION_SCALAR_MUL_I32,
        (InstructionType::Mul(OpDataType::F32), true  ) => INSTRUCTION_VECTOR_CW_MUL_F32,
        (InstructionType::Mul(OpDataType::I32), true  ) => INSTRUCTION_VECTOR_CW_MUL_I32,

        (InstructionType::Div(OpDataType::F32), false ) => INSTRUCTION_SCALAR_DIV_F32,
        (InstructionType::Div(OpDataType::I32), false ) => INSTRUCTION_SCALAR_DIV_I32,
        (InstructionType::Div(OpDataType::F32), true  ) => INSTRUCTION_VECTOR_CW_DIV_F32,
        (InstructionType::Div(OpDataType::I32), true  ) => INSTRUCTION_VECTOR_CW_DIV_I32,

        (InstructionType::Mod(OpDataType::F32), false ) => INSTRUCTION_SCALAR_MOD_F32,
        (InstructionType::Mod(OpDataType::I32), false ) => INSTRUCTION_SCALAR_MOD_I32,
        (InstructionType::Mod(OpDataType::F32), true  ) => INSTRUCTION_VECTOR_CW_MOD_F32,
        (InstructionType::Mod(OpDataType::I32), true  ) => INSTRUCTION_VECTOR_CW_MOD_I32,

        (InstructionType::Atan2,                false ) => INSTRUCTION_SCALAR_ATAN2,
        (InstructionType::Atan2,                true  ) => INSTRUCTION_VECTOR_CW_ATAN2,

        (InstructionType::And,                  false ) => INSTRUCTION_SCALAR_AND,
        (InstructionType::AndN,                 false ) => INSTRUCTION_SCALAR_AND_NOT,
        (InstructionType::Or,                   false ) => INSTRUCTION_SCALAR_OR,
        (InstructionType::Xor,                  false ) => INSTRUCTION_SCALAR_XOR,

        (InstructionType::And,                  true  ) => INSTRUCTION_VECTOR_CW_AND,
        (InstructionType::AndN,                 true  ) => INSTRUCTION_VECTOR_CW_AND_NOT,
        (InstructionType::Or,                   true  ) => INSTRUCTION_VECTOR_CW_OR,
        (InstructionType::Xor,                  true  ) => INSTRUCTION_VECTOR_CW_XOR,

        _ => panic!("Invalid InstructionType passed to write_scalar_binary_op() (internal error)"),
    };
    bytes.push(opcode);
    dst.write(bytes, assembly_mode);
    a.write(bytes, assembly_mode);
    b.write(bytes, assembly_mode);
    Ok(())
}

pub fn write_scalar_unary_op(bytes: &mut Vec<u8>, op_token: &Token, op: InstructionType, dst: RegisterName, src: RegisterName, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
    let vector = match (dst.is_vector(), src.is_vector()) {
        (true, true) => true,
        (false, false) => false,
        _ => {
            Err(SourceError {
                message: format!(
                    "Invalid operand types - dst: {}, src: {}",
                    bool_to_type_name(dst.is_vector()),
                    bool_to_type_name(src.is_vector()),
                ),
                line: op_token.line,
                column: op_token.column,
            })?
        }
    };
    let opcode = match (op, vector) {
        (InstructionType::ConvertF32ToI32,       false) => INSTRUCTION_SCALAR_CONV_F32_TO_I32,
        (InstructionType::ConvertF32ToU32,       false) => INSTRUCTION_SCALAR_CONV_F32_TO_U32,
        (InstructionType::ConvertI32ToF32,       false) => INSTRUCTION_SCALAR_CONV_I32_TO_F32,
        (InstructionType::ConvertU32ToF32,       false) => INSTRUCTION_SCALAR_CONV_U32_TO_F32,
        (InstructionType::ConvertF32ToI32,       true ) => INSTRUCTION_VECTOR_CW_CONV_F32_TO_I32,
        (InstructionType::ConvertF32ToU32,       true ) => INSTRUCTION_VECTOR_CW_CONV_F32_TO_U32,
        (InstructionType::ConvertI32ToF32,       true ) => INSTRUCTION_VECTOR_CW_CONV_I32_TO_F32,
        (InstructionType::ConvertU32ToF32,       true ) => INSTRUCTION_VECTOR_CW_CONV_U32_TO_F32,

        (InstructionType::Neg(OpDataType::F32),  false) => INSTRUCTION_SCALAR_NEG_F32,
        (InstructionType::Neg(OpDataType::I32),  false) => INSTRUCTION_SCALAR_NEG_I32,
        (InstructionType::Neg(OpDataType::F32),  true ) => INSTRUCTION_VECTOR_CW_NEG_F32,
        (InstructionType::Neg(OpDataType::I32),  true ) => INSTRUCTION_VECTOR_CW_NEG_I32,

        (InstructionType::Sign(OpDataType::F32), false) => INSTRUCTION_SCALAR_SIGN_F32,
        (InstructionType::Sign(OpDataType::I32), false) => INSTRUCTION_SCALAR_SIGN_I32,
        (InstructionType::Sign(OpDataType::F32), true ) => INSTRUCTION_VECTOR_CW_SIGN_F32,
        (InstructionType::Sign(OpDataType::I32), true ) => INSTRUCTION_VECTOR_CW_SIGN_I32,

        (InstructionType::Recip,                 false) => INSTRUCTION_SCALAR_RECIP,
        (InstructionType::Sin,                   false) => INSTRUCTION_SCALAR_SIN,
        (InstructionType::Cos,                   false) => INSTRUCTION_SCALAR_COS,
        (InstructionType::Tan,                   false) => INSTRUCTION_SCALAR_TAN,
        (InstructionType::ASin,                  false) => INSTRUCTION_SCALAR_ASIN,
        (InstructionType::ACos,                  false) => INSTRUCTION_SCALAR_ACOS,
        (InstructionType::Atan,                  false) => INSTRUCTION_SCALAR_ATAN,
        (InstructionType::Ln,                    false) => INSTRUCTION_SCALAR_LN,
        (InstructionType::Exp,                   false) => INSTRUCTION_SCALAR_EXP,

        (InstructionType::Recip,                 true ) => INSTRUCTION_VECTOR_CW_RECIP,
        (InstructionType::Sin,                   true ) => INSTRUCTION_VECTOR_CW_SIN,
        (InstructionType::Cos,                   true ) => INSTRUCTION_VECTOR_CW_COS,
        (InstructionType::Tan,                   true ) => INSTRUCTION_VECTOR_CW_TAN,
        (InstructionType::ASin,                  true ) => INSTRUCTION_VECTOR_CW_ASIN,
        (InstructionType::ACos,                  true ) => INSTRUCTION_VECTOR_CW_ACOS,
        (InstructionType::Atan,                  true ) => INSTRUCTION_VECTOR_CW_ATAN,
        (InstructionType::Ln,                    true ) => INSTRUCTION_VECTOR_CW_LN,
        (InstructionType::Exp,                   true ) => INSTRUCTION_VECTOR_CW_EXP,
        _ => panic!("Invalid InstructionType passed to write_scalar_unary_op() (internal error)"),
    };
    bytes.push(opcode);
    dst.write(bytes, assembly_mode);
    src.write(bytes, assembly_mode);
    Ok(())
}