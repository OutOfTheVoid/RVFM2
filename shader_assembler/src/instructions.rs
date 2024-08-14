use super::lexer::*;
use super::src_error::*;

trait WriteInstructionBytes {
    fn write(&self, bytes: &mut Vec<u8>, assembly_mode: AssemblyMode);
}

const REGTYPE_LOCAL    : u8 = 0x00;
const REGTYPE_INPUT    : u8 = 0x01;
const REGTYPE_OUTPUT   : u8 = 0x02;
const REGTYPE_CONSTANT : u8 = 0x03;
const REGTYPE_NONE     : u8 = 0xFF;

impl RegisterName {
    fn write_none(bytes: &mut Vec<u8>) {
        bytes.push(REGTYPE_NONE);
        bytes.push(0x00);
    }
}

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


fn write_offset_u32(bytes: &mut Vec<u8>, offset: u32) {
    bytes.push((offset >>  0) as u8);
    bytes.push((offset >>  8) as u8);
    bytes.push((offset >> 16) as u8);
    bytes.push((offset >> 24) as u8);
}

// ==================================================================== //
// These should match the definitions in rvfm/src/gpu/shader_parser.rs  //
// ==================================================================== //

const OPCODE_VECTOR_PUSH                          : u8 = 0x00;
const OPCODE_SCALAR_PUSH                          : u8 = 0x01;
const OPCODE_VECTOR_POP                           : u8 = 0x02;
const OPCODE_SCALAR_POP                           : u8 = 0x03;
const OPCODE_VECTOR_COPY                          : u8 = 0x04;
const OPCODE_SCALAR_COPY                          : u8 = 0x05;
const OPCODE_VECTOR_COMPONENT_TO_SCALAR_COPY      : u8 = 0x06;
const OPCODE_SCALAR_TO_VECTOR_COMPONENT_COPY      : u8 = 0x07;
const OPCODE_COND_VECTOR_COPY                     : u8 = 0x08;
const OPCODE_COND_SCALAR_COPY                     : u8 = 0x09;
const OPCODE_COND_VECTOR_COMPONENT_TO_SCALAR_COPY : u8 = 0x0A;
const OPCODE_COND_SCALAR_TO_VECTOR_COMPONENT_COPY : u8 = 0x0B;
const OPCODE_COMPARE_SCALAR_F32                   : u8 = 0x0C;
const OPCODE_COMPARE_VECTOR_F32                   : u8 = 0x0D;
const OPCODE_COMPARE_SCALAR_I32                   : u8 = 0x0E;
const OPCODE_COMPARE_VECTOR_I32                   : u8 = 0x0F;
const OPCODE_COMPARE_SCALAR_U32                   : u8 = 0x10;
const OPCODE_COMPARE_VECTOR_U32                   : u8 = 0x11;
const OPCODE_MATRIX_MULTIPLY_M44_V4               : u8 = 0x12;

const OPCODE_SCALAR_ADD_F32                       : u8 = 0x13;
const OPCODE_SCALAR_SUB_F32                       : u8 = 0x14;
const OPCODE_SCALAR_MUL_F32                       : u8 = 0x15;
const OPCODE_SCALAR_DIV_F32                       : u8 = 0x16;
const OPCODE_SCALAR_MOD_F32                       : u8 = 0x17;
const OPCODE_SCALAR_ADD_I32                       : u8 = 0x18;
const OPCODE_SCALAR_SUB_I32                       : u8 = 0x19;
const OPCODE_SCALAR_MUL_I32                       : u8 = 0x1A;
const OPCODE_SCALAR_DIV_I32                       : u8 = 0x1B;
const OPCODE_SCALAR_MOD_I32                       : u8 = 0x1C;

const OPCODE_VECTOR_CW_ADD_F32                    : u8 = 0x1D;
const OPCODE_VECTOR_CW_SUB_F32                    : u8 = 0x1E;
const OPCODE_VECTOR_CW_MUL_F32                    : u8 = 0x1F;
const OPCODE_VECTOR_CW_DIV_F32                    : u8 = 0x20;
const OPCODE_VECTOR_CW_MOD_F32                    : u8 = 0x21;
const OPCODE_VECTOR_CW_ADD_I32                    : u8 = 0x22;
const OPCODE_VECTOR_CW_SUB_I32                    : u8 = 0x23;
const OPCODE_VECTOR_CW_MUL_I32                    : u8 = 0x24;
const OPCODE_VECTOR_CW_DIV_I32                    : u8 = 0x25;
const OPCODE_VECTOR_CW_MOD_I32                    : u8 = 0x26;

const OPCODE_SCALAR_CONV_F32_TO_I32               : u8 = 0x27;
const OPCODE_SCALAR_CONV_F32_TO_U32               : u8 = 0x28;
const OPCODE_SCALAR_CONV_U32_TO_F32               : u8 = 0x29;
const OPCODE_SCALAR_CONV_I32_TO_F32               : u8 = 0x2A;
const OPCODE_SCALAR_NEG_F32                       : u8 = 0x2B;
const OPCODE_SCALAR_NEG_I32                       : u8 = 0x2C;
const OPCODE_SCALAR_SIGN_F32                      : u8 = 0x2D;
const OPCODE_SCALAR_SIGN_I32                      : u8 = 0x2E;
const OPCODE_SCALAR_RECIP                         : u8 = 0x2F;
const OPCODE_SCALAR_SIN                           : u8 = 0x30;
const OPCODE_SCALAR_COS                           : u8 = 0x31;
const OPCODE_SCALAR_TAN                           : u8 = 0x32;
const OPCODE_SCALAR_ASIN                          : u8 = 0x33;
const OPCODE_SCALAR_ACOS                          : u8 = 0x34;
const OPCODE_SCALAR_ATAN                          : u8 = 0x35;
const OPCODE_SCALAR_LN                            : u8 = 0x36;
const OPCODE_SCALAR_EXP                           : u8 = 0x37;

const OPCODE_VECTOR_CW_CONV_F32_TO_I32               : u8 = 0x38;
const OPCODE_VECTOR_CW_CONV_F32_TO_U32               : u8 = 0x39;
const OPCODE_VECTOR_CW_CONV_U32_TO_F32               : u8 = 0x3A;
const OPCODE_VECTOR_CW_CONV_I32_TO_F32               : u8 = 0x3B;
const OPCODE_VECTOR_CW_NEG_F32                       : u8 = 0x3C;
const OPCODE_VECTOR_CW_NEG_I32                       : u8 = 0x3D;
const OPCODE_VECTOR_CW_SIGN_F32                      : u8 = 0x3E;
const OPCODE_VECTOR_CW_SIGN_I32                      : u8 = 0x3F;
const OPCODE_VECTOR_CW_RECIP                         : u8 = 0x40;
const OPCODE_VECTOR_CW_SIN                           : u8 = 0x41;
const OPCODE_VECTOR_CW_COS                           : u8 = 0x42;
const OPCODE_VECTOR_CW_TAN                           : u8 = 0x43;
const OPCODE_VECTOR_CW_ASIN                          : u8 = 0x44;
const OPCODE_VECTOR_CW_ACOS                          : u8 = 0x45;
const OPCODE_VECTOR_CW_ATAN                          : u8 = 0x46;
const OPCODE_VECTOR_CW_LN                            : u8 = 0x47;
const OPCODE_VECTOR_CW_EXP                           : u8 = 0x48;

const OPCODE_SCALAR_ATAN2                            : u8 = 0x49;
const OPCODE_SCALAR_AND                              : u8 = 0x4A;
const OPCODE_SCALAR_AND_NOT                          : u8 = 0x4B;
const OPCODE_SCALAR_OR                               : u8 = 0x4C;
const OPCODE_SCALAR_XOR                              : u8 = 0x4D;

const OPCODE_VECTOR_CW_ATAN2                         : u8 = 0x4E;
const OPCODE_VECTOR_CW_AND                           : u8 = 0x4F;
const OPCODE_VECTOR_CW_AND_NOT                       : u8 = 0x50;
const OPCODE_VECTOR_CW_OR                            : u8 = 0x51;
const OPCODE_VECTOR_CW_XOR                           : u8 = 0x52;

const OPCODE_NORM2                                   : u8 = 0x53;
const OPCODE_NORM3                                   : u8 = 0x54;
const OPCODE_NORM4                                   : u8 = 0x55;
const OPCODE_MAG2                                    : u8 = 0x56;
const OPCODE_MAG3                                    : u8 = 0x57;
const OPCODE_MAG4                                    : u8 = 0x58;
const OPCODE_SQ_MAG2                                 : u8 = 0x59;
const OPCODE_SQ_MAG3                                 : u8 = 0x5A;
const OPCODE_SQ_MAG4                                 : u8 = 0x5B;

const OPCODE_CROSS                                   : u8 = 0x5C;

const OPCODE_FMA_SCALAR_F32                          : u8 = 0x5D;
const OPCODE_FMA_SCALAR_I32                          : u8 = 0x5E;

const OPCODE_BUFFER_READ_SCALAR                      : u8 = 0x5F;

pub fn write_push(bytes: &mut Vec<u8>, src: RegisterName, _src_token: &Token, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
    bytes.push(if src.is_vector() { OPCODE_VECTOR_PUSH } else { OPCODE_SCALAR_PUSH });
    src.write(bytes, assembly_mode);
    Ok(())
}

pub fn write_pop(bytes: &mut Vec<u8>, dst: RegisterName, _dst_token: &Token, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
    bytes.push(if dst.is_vector() { OPCODE_VECTOR_POP } else { OPCODE_SCALAR_POP });
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
            bytes.push(OPCODE_VECTOR_COPY);
            dst.write(bytes, assembly_mode);
            src.write(bytes, assembly_mode);
        },

        (false, None,            false, None           ) => {
            bytes.push(OPCODE_SCALAR_COPY);
            dst.write(bytes, assembly_mode);
            src.write(bytes, assembly_mode);
        },

        (false, None,            true,  Some(component)) => {
            let component = Component::from_token(component)?;
            bytes.push(OPCODE_VECTOR_COMPONENT_TO_SCALAR_COPY);
            dst.write(bytes, assembly_mode);
            src.write(bytes, assembly_mode);
            component.write(bytes, assembly_mode);
        },

        (true,  Some(component), false, None           ) => {
            let component = Component::from_token(component)?;
            bytes.push(OPCODE_SCALAR_TO_VECTOR_COMPONENT_COPY);
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
            bytes.push(OPCODE_COND_VECTOR_COPY);
            cond.write(bytes, assembly_mode);
            dst.write(bytes, assembly_mode);
            src.write(bytes, assembly_mode);
        },

        (false, None,            false, None           ) => {
            bytes.push(OPCODE_COND_SCALAR_COPY);
            cond.write(bytes, assembly_mode);
            dst.write(bytes, assembly_mode);
            src.write(bytes, assembly_mode);
        },

        (false, None,            true,  Some(component)) => {
            let component = Component::from_token(component)?;
            bytes.push(OPCODE_COND_VECTOR_COMPONENT_TO_SCALAR_COPY);
            cond.write(bytes, assembly_mode);
            dst.write(bytes, assembly_mode);
            src.write(bytes, assembly_mode);
            component.write(bytes, assembly_mode);
        },

        (true,  Some(component), false, None           ) => {
            println!("writing OPCODE_COND_SCALAR_TO_VECTOR_COMPONENT_COPY");
            let component = Component::from_token(component)?;
            bytes.push(OPCODE_COND_SCALAR_TO_VECTOR_COMPONENT_COPY);
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
        (true, true, true)    => OPCODE_COMPARE_VECTOR_F32,
        (false, false, false) => OPCODE_COMPARE_SCALAR_F32,
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
        (true, true, true)    => OPCODE_COMPARE_VECTOR_I32,
        (false, false, false) => OPCODE_COMPARE_SCALAR_I32,
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
        (true, true, true)    => OPCODE_COMPARE_VECTOR_U32,
        (false, false, false) => OPCODE_COMPARE_SCALAR_U32,
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
    bytes.push(OPCODE_MATRIX_MULTIPLY_M44_V4);
    dst.write(bytes, assembly_mode);
    a0.write(bytes, assembly_mode);
    a1.write(bytes, assembly_mode);
    a2.write(bytes, assembly_mode);
    a3.write(bytes, assembly_mode);
    x.write(bytes, assembly_mode);
    Ok(())
}

pub fn write_binary_op(bytes: &mut Vec<u8>, op_token: &Token, op: InstructionType, dst: RegisterName, a: RegisterName, b: RegisterName, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
    let operand_types = (dst.is_vector(), a.is_vector(), b.is_vector());
    const OT_ALL_SCALAR: (bool, bool, bool) = (true, true, true);
    const OT_ALL_VECTOR: (bool, bool, bool) = (false, false, false);
    let opcode = match (op, operand_types) {
        (InstructionType::Add(OpDataType::F32), OT_ALL_SCALAR) => OPCODE_SCALAR_ADD_F32,
        (InstructionType::Add(OpDataType::I32), OT_ALL_SCALAR) => OPCODE_SCALAR_ADD_I32,
        (InstructionType::Add(OpDataType::F32), OT_ALL_VECTOR) => OPCODE_VECTOR_CW_ADD_F32,
        (InstructionType::Add(OpDataType::I32), OT_ALL_VECTOR) => OPCODE_VECTOR_CW_ADD_I32,

        (InstructionType::Sub(OpDataType::F32), OT_ALL_SCALAR) => OPCODE_SCALAR_SUB_F32,
        (InstructionType::Sub(OpDataType::I32), OT_ALL_SCALAR) => OPCODE_SCALAR_SUB_I32,
        (InstructionType::Sub(OpDataType::F32), OT_ALL_VECTOR) => OPCODE_VECTOR_CW_SUB_F32,
        (InstructionType::Sub(OpDataType::I32), OT_ALL_VECTOR) => OPCODE_VECTOR_CW_SUB_I32,

        (InstructionType::Mul(OpDataType::F32), OT_ALL_SCALAR) => OPCODE_SCALAR_MUL_F32,
        (InstructionType::Mul(OpDataType::I32), OT_ALL_SCALAR) => OPCODE_SCALAR_MUL_I32,
        (InstructionType::Mul(OpDataType::F32), OT_ALL_VECTOR) => OPCODE_VECTOR_CW_MUL_F32,
        (InstructionType::Mul(OpDataType::I32), OT_ALL_VECTOR) => OPCODE_VECTOR_CW_MUL_I32,

        (InstructionType::Div(OpDataType::F32), OT_ALL_SCALAR) => OPCODE_SCALAR_DIV_F32,
        (InstructionType::Div(OpDataType::I32), OT_ALL_SCALAR) => OPCODE_SCALAR_DIV_I32,
        (InstructionType::Div(OpDataType::F32), OT_ALL_VECTOR) => OPCODE_VECTOR_CW_DIV_F32,
        (InstructionType::Div(OpDataType::I32), OT_ALL_VECTOR) => OPCODE_VECTOR_CW_DIV_I32,

        (InstructionType::Mod(OpDataType::F32), OT_ALL_SCALAR) => OPCODE_SCALAR_MOD_F32,
        (InstructionType::Mod(OpDataType::I32), OT_ALL_SCALAR) => OPCODE_SCALAR_MOD_I32,
        (InstructionType::Mod(OpDataType::F32), OT_ALL_VECTOR) => OPCODE_VECTOR_CW_MOD_F32,
        (InstructionType::Mod(OpDataType::I32), OT_ALL_VECTOR) => OPCODE_VECTOR_CW_MOD_I32,

        (InstructionType::Atan2,                OT_ALL_SCALAR) => OPCODE_SCALAR_ATAN2,
        (InstructionType::Atan2,                OT_ALL_VECTOR) => OPCODE_VECTOR_CW_ATAN2,

        (InstructionType::And,                  OT_ALL_SCALAR) => OPCODE_SCALAR_AND,
        (InstructionType::AndN,                 OT_ALL_SCALAR) => OPCODE_SCALAR_AND_NOT,
        (InstructionType::Or,                   OT_ALL_SCALAR) => OPCODE_SCALAR_OR,
        (InstructionType::Xor,                  OT_ALL_SCALAR) => OPCODE_SCALAR_XOR,

        (InstructionType::And,                  OT_ALL_VECTOR) => OPCODE_VECTOR_CW_AND,
        (InstructionType::AndN,                 OT_ALL_VECTOR) => OPCODE_VECTOR_CW_AND_NOT,
        (InstructionType::Or,                   OT_ALL_VECTOR) => OPCODE_VECTOR_CW_OR,
        (InstructionType::Xor,                  OT_ALL_VECTOR) => OPCODE_VECTOR_CW_XOR,

        (InstructionType::Cross,                OT_ALL_VECTOR) => OPCODE_CROSS,

        (InstructionType::Add(_),               _            ) |
        (InstructionType::Sub(_),               _            ) |
        (InstructionType::Div(_),               _            ) |
        (InstructionType::Mul(_),               _            ) |
        (InstructionType::Mod(_),               _            ) |
        (InstructionType::Atan2,                _            ) |
        (InstructionType::And,                  _            ) |
        (InstructionType::Or,                   _            ) |
        (InstructionType::Xor,                  _            ) |
        (InstructionType::AndN,                 _            ) |
        (InstructionType::Cross,                _            ) => {
            Err(SourceError {
                message: format!("Invalid operand types for instruction: {:?}: {}, {}, {}", op, bool_to_type_name(dst.is_vector()), bool_to_type_name(a.is_vector()), bool_to_type_name(b.is_vector())),
                line: op_token.line,
                column: op_token.column,
            })?
        },
        _ => panic!("Invalid InstructionType passed to write_scalar_binary_op() (internal error)"),
    };
    bytes.push(opcode);
    dst.write(bytes, assembly_mode);
    a.write(bytes, assembly_mode);
    b.write(bytes, assembly_mode);
    Ok(())
}

pub fn write_unary_op(bytes: &mut Vec<u8>, op_token: &Token, op: InstructionType, dst: RegisterName, src: RegisterName, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
    let operand_types = (dst.is_vector(), src.is_vector());
    const OT_ALL_SCALAR: (bool, bool) = (false, false);
    const OT_ALL_VECTOR: (bool, bool) = (true,  true );
    const OT_VEC_TO_SCL: (bool, bool) = (false, true );
    let opcode = match (op, operand_types) {
        (InstructionType::ConvertF32ToI32,       OT_ALL_SCALAR) => OPCODE_SCALAR_CONV_F32_TO_I32,
        (InstructionType::ConvertF32ToU32,       OT_ALL_SCALAR) => OPCODE_SCALAR_CONV_F32_TO_U32,
        (InstructionType::ConvertI32ToF32,       OT_ALL_SCALAR) => OPCODE_SCALAR_CONV_I32_TO_F32,
        (InstructionType::ConvertU32ToF32,       OT_ALL_SCALAR) => OPCODE_SCALAR_CONV_U32_TO_F32,
        (InstructionType::ConvertF32ToI32,       OT_ALL_VECTOR) => OPCODE_VECTOR_CW_CONV_F32_TO_I32,
        (InstructionType::ConvertF32ToU32,       OT_ALL_VECTOR) => OPCODE_VECTOR_CW_CONV_F32_TO_U32,
        (InstructionType::ConvertI32ToF32,       OT_ALL_VECTOR) => OPCODE_VECTOR_CW_CONV_I32_TO_F32,
        (InstructionType::ConvertU32ToF32,       OT_ALL_VECTOR) => OPCODE_VECTOR_CW_CONV_U32_TO_F32,

        (InstructionType::Neg(OpDataType::F32),  OT_ALL_SCALAR) => OPCODE_SCALAR_NEG_F32,
        (InstructionType::Neg(OpDataType::I32),  OT_ALL_SCALAR) => OPCODE_SCALAR_NEG_I32,
        (InstructionType::Neg(OpDataType::F32),  OT_ALL_VECTOR) => OPCODE_VECTOR_CW_NEG_F32,
        (InstructionType::Neg(OpDataType::I32),  OT_ALL_VECTOR) => OPCODE_VECTOR_CW_NEG_I32,

        (InstructionType::Sign(OpDataType::F32), OT_ALL_SCALAR) => OPCODE_SCALAR_SIGN_F32,
        (InstructionType::Sign(OpDataType::I32), OT_ALL_SCALAR) => OPCODE_SCALAR_SIGN_I32,
        (InstructionType::Sign(OpDataType::F32), OT_ALL_VECTOR) => OPCODE_VECTOR_CW_SIGN_F32,
        (InstructionType::Sign(OpDataType::I32), OT_ALL_VECTOR) => OPCODE_VECTOR_CW_SIGN_I32,

        (InstructionType::Recip,                 OT_ALL_SCALAR) => OPCODE_SCALAR_RECIP,
        (InstructionType::Sin,                   OT_ALL_SCALAR) => OPCODE_SCALAR_SIN,
        (InstructionType::Cos,                   OT_ALL_SCALAR) => OPCODE_SCALAR_COS,
        (InstructionType::Tan,                   OT_ALL_SCALAR) => OPCODE_SCALAR_TAN,
        (InstructionType::ASin,                  OT_ALL_SCALAR) => OPCODE_SCALAR_ASIN,
        (InstructionType::ACos,                  OT_ALL_SCALAR) => OPCODE_SCALAR_ACOS,
        (InstructionType::Atan,                  OT_ALL_SCALAR) => OPCODE_SCALAR_ATAN,
        (InstructionType::Ln,                    OT_ALL_SCALAR) => OPCODE_SCALAR_LN,
        (InstructionType::Exp,                   OT_ALL_SCALAR) => OPCODE_SCALAR_EXP,

        (InstructionType::Recip,                 OT_ALL_VECTOR) => OPCODE_VECTOR_CW_RECIP,
        (InstructionType::Sin,                   OT_ALL_VECTOR) => OPCODE_VECTOR_CW_SIN,
        (InstructionType::Cos,                   OT_ALL_VECTOR) => OPCODE_VECTOR_CW_COS,
        (InstructionType::Tan,                   OT_ALL_VECTOR) => OPCODE_VECTOR_CW_TAN,
        (InstructionType::ASin,                  OT_ALL_VECTOR) => OPCODE_VECTOR_CW_ASIN,
        (InstructionType::ACos,                  OT_ALL_VECTOR) => OPCODE_VECTOR_CW_ACOS,
        (InstructionType::Atan,                  OT_ALL_VECTOR) => OPCODE_VECTOR_CW_ATAN,
        (InstructionType::Ln,                    OT_ALL_VECTOR) => OPCODE_VECTOR_CW_LN,
        (InstructionType::Exp,                   OT_ALL_VECTOR) => OPCODE_VECTOR_CW_EXP,

        (InstructionType::Norm2,                 OT_ALL_VECTOR) => OPCODE_NORM2,
        (InstructionType::Norm3,                 OT_ALL_VECTOR) => OPCODE_NORM3,
        (InstructionType::Norm4,                 OT_ALL_VECTOR) => OPCODE_NORM4,

        (InstructionType::Mag2,                  OT_VEC_TO_SCL) => OPCODE_MAG2,
        (InstructionType::Mag3,                  OT_VEC_TO_SCL) => OPCODE_MAG3,
        (InstructionType::Mag4,                  OT_VEC_TO_SCL) => OPCODE_MAG4,
        (InstructionType::SqMag2,                OT_VEC_TO_SCL) => OPCODE_SQ_MAG2,
        (InstructionType::SqMag3,                OT_VEC_TO_SCL) => OPCODE_SQ_MAG3,
        (InstructionType::SqMag4,                OT_VEC_TO_SCL) => OPCODE_SQ_MAG4,

        (InstructionType::ConvertF32ToI32,       _            ) |
        (InstructionType::ConvertF32ToU32,       _            ) |
        (InstructionType::ConvertI32ToF32,       _            ) |
        (InstructionType::ConvertU32ToF32,       _            ) |
        (InstructionType::Neg(OpDataType::F32),  _            ) |
        (InstructionType::Neg(OpDataType::I32),  _            ) |
        (InstructionType::Sign(OpDataType::F32), _            ) |
        (InstructionType::Sign(OpDataType::I32), _            ) |
        (InstructionType::Recip,                 _            ) |
        (InstructionType::Sin,                   _            ) |
        (InstructionType::Cos,                   _            ) |
        (InstructionType::Tan,                   _            ) |
        (InstructionType::ASin,                  _            ) |
        (InstructionType::ACos,                  _            ) |
        (InstructionType::Atan,                  _            ) |
        (InstructionType::Ln,                    _            ) |
        (InstructionType::Exp,                   _            ) |
        (InstructionType::Norm2,                 _            ) |
        (InstructionType::Norm3,                 _            ) |
        (InstructionType::Norm4,                 _            ) |
        (InstructionType::Mag2,                  _            ) |
        (InstructionType::Mag3,                  _            ) |
        (InstructionType::Mag4,                  _            ) |
        (InstructionType::SqMag2,                _            ) |
        (InstructionType::SqMag3,                _            ) |
        (InstructionType::SqMag4,                _            ) => {
            Err(SourceError {
                message: format!("Invalid operand types for instruction: {:?}: {}, {}", op, bool_to_type_name(dst.is_vector()), bool_to_type_name(src.is_vector())),
                line: op_token.line,
                column: op_token.column,
            })?
        },

        _ => panic!("Invalid InstructionType passed to write_scalar_unary_op() (internal error)"),
    };
    bytes.push(opcode);
    dst.write(bytes, assembly_mode);
    src.write(bytes, assembly_mode);
    Ok(())
}

pub fn write_ternary_op(bytes: &mut Vec<u8>, op_token: &Token, op: InstructionType, dst: RegisterName, src_a: RegisterName, src_b: RegisterName, src_c: RegisterName, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
    let operand_types = (dst.is_vector(), src_a.is_vector(), src_b.is_vector(), src_c.is_vector());
    const OT_ALL_VECTOR: (bool, bool, bool, bool) = (true, true, true, true);
    const OT_ALL_SCALAR: (bool, bool, bool, bool) = (false, false, false, false); 
    const OT_LERP:       (bool, bool, bool, bool) = (true, true, true, false); 
    let opcode = match (op, operand_types) {
        (InstructionType::Fma(OpDataType::F32), OT_ALL_SCALAR)=> todo!(),
        (InstructionType::Fma(OpDataType::I32), OT_ALL_SCALAR)=> todo!(),
        (InstructionType::Fma(OpDataType::F32), OT_ALL_VECTOR)=> todo!(),
        (InstructionType::Fma(OpDataType::I32), OT_ALL_VECTOR)=> todo!(),
        (InstructionType::Fma(_), _) => {
            Err(SourceError {
                message: format!(
                    "Invalid operand types for fma - dst: {}, src_a: {}, src_b: {}, src_c: {}",
                    bool_to_type_name(operand_types.0),
                    bool_to_type_name(operand_types.1),
                    bool_to_type_name(operand_types.2),
                    bool_to_type_name(operand_types.3),
                ),
                line: op_token.line,
                column: op_token.column,
            })?
        },
        (InstructionType::Lerp, OT_LERP) => todo!(),
        _ => panic!("Invalid InstructionType passed to write_ternary_op() (internal error)")
    };
    bytes.push(opcode);
    dst.write(bytes, assembly_mode);
    src_a.write(bytes, assembly_mode);
    src_b.write(bytes, assembly_mode);
    src_c.write(bytes, assembly_mode);
    Ok(())
}

pub fn write_buffer_read_op(bytes: &mut Vec<u8>, _op_token: &Token, _op: InstructionType, data_type: BufferDataType, dst: RegisterName, dst_token: &Token, src_addr_u32: Option<RegisterName>, offset: u32, buffer: u8, assembly_mode: AssemblyMode) -> Result<(), SourceError> {
    let vector = dst.is_vector();
    if vector {
        Err(SourceError {
            message: format!("Expected scalar register in scalar buffer read"),
            line: dst_token.line,
            column: dst_token.column,
        })?
    }
    bytes.push(OPCODE_BUFFER_READ_SCALAR);
    bytes.push(data_type.to_u8());
    dst.write(bytes, assembly_mode);
    write_offset_u32(bytes, offset);
    bytes.push(buffer);
    if let Some(addr_src_reg) = src_addr_u32 {
        addr_src_reg.write(bytes, assembly_mode);
    } else {
        RegisterName::write_none(bytes);
    }
    Ok(())
}
