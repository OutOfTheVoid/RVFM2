use std::marker::PhantomData;

use super::shader::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShaderParseError {
    UnexpectedEndOfCode,
    InvalidRegisterAddress(usize),
    UnknownOpcode,
    ShaderTooLong,
    InvalidComparison,
}

// ========================================================================== //
// These should match the definitions in shader_assembler/src/instructions.rs //
// ========================================================================== //

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

pub fn get_byte(i: &mut usize, code: &[u8]) -> Result<u8, ShaderParseError> {
    if *i >= code.len() {
        Err(ShaderParseError::UnexpectedEndOfCode)
    } else {
        let t = *i;
        *i += 1;
        Ok(code[t])
    }
}

const REGTYPE_LOCAL    : u8 = 0x00;
const REGTYPE_INPUT    : u8 = 0x01;
const REGTYPE_OUTPUT   : u8 = 0x02;
const REGTYPE_CONSTANT : u8 = 0x03;

pub fn get_register<T: RegisterType>(i: &mut usize, code: &[u8]) -> Result<RegisterAddress<T>, ShaderParseError> {
    let offset = *i;
    let t = get_byte(i, code)?;
    let index = get_byte(i, code)?;
    Ok(match t {
        REGTYPE_LOCAL    => RegisterAddress::Local   (index, PhantomData),
        REGTYPE_INPUT    => RegisterAddress::Input   (index, PhantomData),
        REGTYPE_OUTPUT   => RegisterAddress::Output  (index, PhantomData),
        REGTYPE_CONSTANT => RegisterAddress::Constant(index, PhantomData),
        _ => Err(ShaderParseError::InvalidRegisterAddress(offset))?
    })
}

const VECTOR_CHANNEL_X: u8 = 0;
const VECTOR_CHANNEL_Y: u8 = 1;
const VECTOR_CHANNEL_Z: u8 = 2;
const VECTOR_CHANNEL_W: u8 = 3;

pub fn get_vector_channel(i: &mut usize, code: &[u8]) -> Result<VectorChannel, ShaderParseError> {
    let offset = *i;
    let c = get_byte(i, code)?;
    Ok(match c {
        VECTOR_CHANNEL_X => VectorChannel::X,
        VECTOR_CHANNEL_Y => VectorChannel::Y,
        VECTOR_CHANNEL_Z => VectorChannel::Z,
        VECTOR_CHANNEL_W => VectorChannel::W,
        _ => Err(ShaderParseError::InvalidRegisterAddress(offset))?
    })
}

pub fn parse_shader_bytecode(shader_type: ShaderType, code: &[u8], module: &mut ShaderModule) -> Result<(), ShaderParseError> {
    let mut instruction = 0;
    let mut i = 0;
    while i < code.len() {
        if instruction >= module.instruction_buffer.len() {
            Err(ShaderParseError::ShaderTooLong)?
        }
        let opcode = code[i];
        i += 1;
        match opcode {
            OPCODE_VECTOR_PUSH => {
                let src = get_register::<Vector>(&mut i, code)?;
                module.instruction_buffer[instruction] = ShaderInstruction::PushVector(src);
            },
            OPCODE_SCALAR_PUSH => {
                let src = get_register::<Scalar>(&mut i, code)?;
                module.instruction_buffer[instruction] = ShaderInstruction::PushScalar(src);
            },
            OPCODE_VECTOR_POP => {
                let dst = get_register::<Vector>(&mut i, code)?;
                module.instruction_buffer[instruction] = ShaderInstruction::PopVector(dst);
            },
            OPCODE_SCALAR_POP => {
                let dst = get_register::<Scalar>(&mut i, code)?;
                module.instruction_buffer[instruction] = ShaderInstruction::PopScalar(dst);
            },
            OPCODE_VECTOR_COPY => {
                let to = get_register::<Vector>(&mut i, code)?;
                let from = get_register::<Vector>(&mut i, code)?;
                module.instruction_buffer[instruction] = ShaderInstruction::CopyVectorRegister { from, to };
            },
            OPCODE_SCALAR_COPY => {
                let to = get_register::<Scalar>(&mut i, code)?;
                let from = get_register::<Scalar>(&mut i, code)?;
                module.instruction_buffer[instruction] = ShaderInstruction::CopyScalarRegister { from, to };
            },
            OPCODE_VECTOR_COMPONENT_TO_SCALAR_COPY => {
                let to = get_register::<Scalar>(&mut i, code)?;
                let from = get_register::<Vector>(&mut i, code)?;
                let channel = get_vector_channel(&mut i, code)?;
                module.instruction_buffer[instruction] = ShaderInstruction::CopyVectorComponentToScalar { channel, from, to };
            },
            OPCODE_SCALAR_TO_VECTOR_COMPONENT_COPY => {
                let to = get_register::<Vector>(&mut i, code)?;
                let channel = get_vector_channel(&mut i, code)?;
                let from = get_register::<Scalar>(&mut i, code)?;
                module.instruction_buffer[instruction] = ShaderInstruction::CopyScalarToVectorComponent { channel, from, to };
            },
            OPCODE_COND_VECTOR_COPY => {
                let cond = get_register::<Scalar>(&mut i, code)?;
                let to = get_register::<Vector>(&mut i, code)?;
                let from = get_register::<Vector>(&mut i, code)?;
                module.instruction_buffer[instruction] = ShaderInstruction::ConditionallyCopyVectorRegister { cond, from, to };
            },
            OPCODE_COND_SCALAR_COPY => {
                let cond = get_register::<Scalar>(&mut i, code)?;
                let to = get_register::<Scalar>(&mut i, code)?;
                let from = get_register::<Scalar>(&mut i, code)?;
                module.instruction_buffer[instruction] = ShaderInstruction::ConditionallyCopyScalarRegister { cond, from, to };
            },
            OPCODE_COND_VECTOR_COMPONENT_TO_SCALAR_COPY => {
                let cond = get_register::<Scalar>(&mut i, code)?;
                let to = get_register::<Scalar>(&mut i, code)?;
                let from = get_register::<Vector>(&mut i, code)?;
                let channel = get_vector_channel(&mut i, code)?;
                module.instruction_buffer[instruction] = ShaderInstruction::ConditionallyCopyVectorComponentToScalar { cond, channel, from, to };
            },
            OPCODE_COND_SCALAR_TO_VECTOR_COMPONENT_COPY => {
                let cond = get_register::<Scalar>(&mut i, code)?;
                let to = get_register::<Vector>(&mut i, code)?;
                let channel = get_vector_channel(&mut i, code)?;
                let from = get_register::<Scalar>(&mut i, code)?;
                module.instruction_buffer[instruction] = ShaderInstruction::ConditionallyCopyScalarToVectorComponent { cond, channel, from, to };
            },
            OPCODE_COMPARE_SCALAR_F32 |
            OPCODE_COMPARE_SCALAR_I32 |
            OPCODE_COMPARE_SCALAR_U32 => {
                let dst = get_register::<Scalar>(&mut i, code)?;
                let src_a = get_register::<Scalar>(&mut i, code)?;
                let src_b = get_register::<Scalar>(&mut i, code)?;
                let comparison = Comparison::from_u8(get_byte(&mut i, code)?)
                    .map(|x| Ok(x))
                    .unwrap_or_else(|| Err(ShaderParseError::InvalidComparison))?;
                module.instruction_buffer[instruction] = match opcode {
                    OPCODE_COMPARE_SCALAR_F32 => {
                        ShaderInstruction::ScalarBinaryOp {
                            src_a,
                            src_b,
                            dst,
                            op: ScalarBinaryOp::Compare(OpDataType::F32, comparison)
                        }
                    }
                    OPCODE_COMPARE_SCALAR_I32 => {
                        ShaderInstruction::ScalarBinaryOp {
                            src_a,
                            src_b,
                            dst,
                            op: ScalarBinaryOp::Compare(OpDataType::I32, comparison)
                        }
                    },
                    OPCODE_COMPARE_SCALAR_U32 => {
                        ShaderInstruction::ScalarBinaryOp {
                            src_a,
                            src_b,
                            dst,
                            op: ScalarBinaryOp::CompareUnsigned(comparison)
                        }
                    },
                    _ => unreachable!()
                };
            },
            OPCODE_COMPARE_VECTOR_F32 |
            OPCODE_COMPARE_VECTOR_I32 |
            OPCODE_COMPARE_VECTOR_U32 => {
                let dst = get_register::<Vector>(&mut i, code)?;
                let src_a = get_register::<Vector>(&mut i, code)?;
                let src_b = get_register::<Vector>(&mut i, code)?;
                let comparison = Comparison::from_u8(get_byte(&mut i, code)?)
                    .map(|x| Ok(x))
                    .unwrap_or_else(|| Err(ShaderParseError::InvalidComparison))?;
                module.instruction_buffer[instruction] = match opcode {
                    OPCODE_COMPARE_SCALAR_F32 => {
                        ShaderInstruction::VectorComponentwiseScalarBinaryOp {
                            src_a,
                            src_b,
                            dst,
                            op: ScalarBinaryOp::Compare(OpDataType::F32, comparison)
                        }
                    }
                    OPCODE_COMPARE_SCALAR_I32 => {
                        ShaderInstruction::VectorComponentwiseScalarBinaryOp {
                            src_a,
                            src_b,
                            dst,
                            op: ScalarBinaryOp::Compare(OpDataType::I32, comparison)
                        }
                    },
                    OPCODE_COMPARE_SCALAR_U32 => {
                        ShaderInstruction::VectorComponentwiseScalarBinaryOp {
                            src_a,
                            src_b,
                            dst,
                            op: ScalarBinaryOp::CompareUnsigned(comparison)
                        }
                    },
                    _ => unreachable!()
                };
            },
            OPCODE_MATRIX_MULTIPLY_M44_V4 => {
                let dest = get_register::<Vector>(&mut i, code)?;
                let a0 = get_register::<Vector>(&mut i, code)?;
                let a1 = get_register::<Vector>(&mut i, code)?;
                let a2 = get_register::<Vector>(&mut i, code)?;
                let a3 = get_register::<Vector>(&mut i, code)?;
                let x = get_register::<Vector>(&mut i, code)?;
                module.instruction_buffer[instruction] = ShaderInstruction::MatrixMultiply4x4V4 {
                    a0, a1, a2, a3,
                    x,
                    dest
                };
            },

            OPCODE_SCALAR_ADD_F32 |  
            OPCODE_SCALAR_SUB_F32 |  
            OPCODE_SCALAR_MUL_F32 |  
            OPCODE_SCALAR_DIV_F32 |  
            OPCODE_SCALAR_MOD_F32 |  
            OPCODE_SCALAR_ADD_I32 |  
            OPCODE_SCALAR_SUB_I32 |  
            OPCODE_SCALAR_MUL_I32 |  
            OPCODE_SCALAR_DIV_I32 |  
            OPCODE_SCALAR_MOD_I32 |
            OPCODE_SCALAR_ATAN2   |
			OPCODE_SCALAR_AND     |
			OPCODE_SCALAR_AND_NOT |
			OPCODE_SCALAR_OR      |
			OPCODE_SCALAR_XOR     => {
                let dst = get_register::<Scalar>(&mut i, code)?;
                let src_a = get_register::<Scalar>(&mut i, code)?;
                let src_b = get_register::<Scalar>(&mut i, code)?;
                let op = match opcode {
                    OPCODE_SCALAR_ADD_F32 => ScalarBinaryOp::Add     (OpDataType::F32),
                    OPCODE_SCALAR_SUB_F32 => ScalarBinaryOp::Subtract(OpDataType::F32),
                    OPCODE_SCALAR_MUL_F32 => ScalarBinaryOp::Multiply(OpDataType::F32),
                    OPCODE_SCALAR_DIV_F32 => ScalarBinaryOp::Divide  (OpDataType::F32),
                    OPCODE_SCALAR_MOD_F32 => ScalarBinaryOp::Modulo  (OpDataType::F32),
                    OPCODE_SCALAR_ADD_I32 => ScalarBinaryOp::Add     (OpDataType::I32),
                    OPCODE_SCALAR_SUB_I32 => ScalarBinaryOp::Subtract(OpDataType::I32),
                    OPCODE_SCALAR_MUL_I32 => ScalarBinaryOp::Multiply(OpDataType::I32),
                    OPCODE_SCALAR_DIV_I32 => ScalarBinaryOp::Divide  (OpDataType::I32),
                    OPCODE_SCALAR_MOD_I32 => ScalarBinaryOp::Modulo  (OpDataType::I32),
                    OPCODE_SCALAR_ATAN2   => ScalarBinaryOp::Atan2                    ,
                    OPCODE_SCALAR_AND     => ScalarBinaryOp::And                      ,
                    OPCODE_SCALAR_AND_NOT => ScalarBinaryOp::AndNot                   ,
                    OPCODE_SCALAR_OR      => ScalarBinaryOp::Or                       ,
                    OPCODE_SCALAR_XOR     => ScalarBinaryOp::Xor                      ,
                    _ => unreachable!()
                };
                module.instruction_buffer[instruction] = ShaderInstruction::ScalarBinaryOp {
                    src_a,
                    src_b,
                    dst,
                    op
                };
            },

            OPCODE_VECTOR_CW_ADD_F32 |  
            OPCODE_VECTOR_CW_SUB_F32 |  
            OPCODE_VECTOR_CW_MUL_F32 |  
            OPCODE_VECTOR_CW_DIV_F32 |  
            OPCODE_VECTOR_CW_MOD_F32 |  
            OPCODE_VECTOR_CW_ADD_I32 |  
            OPCODE_VECTOR_CW_SUB_I32 |  
            OPCODE_VECTOR_CW_MUL_I32 |  
            OPCODE_VECTOR_CW_DIV_I32 |  
            OPCODE_VECTOR_CW_MOD_I32 |
			OPCODE_VECTOR_CW_ATAN2   |
			OPCODE_VECTOR_CW_AND     |
			OPCODE_VECTOR_CW_AND_NOT |
			OPCODE_VECTOR_CW_OR      |
			OPCODE_VECTOR_CW_XOR     => {
                let dst = get_register::<Vector>(&mut i, code)?;
                let src_a = get_register::<Vector>(&mut i, code)?;
                let src_b = get_register::<Vector>(&mut i, code)?;
                let op = match opcode {
                    OPCODE_VECTOR_CW_ADD_F32 => ScalarBinaryOp::Add     (OpDataType::F32),
                    OPCODE_VECTOR_CW_SUB_F32 => ScalarBinaryOp::Subtract(OpDataType::F32),
                    OPCODE_VECTOR_CW_MUL_F32 => ScalarBinaryOp::Multiply(OpDataType::F32),
                    OPCODE_VECTOR_CW_DIV_F32 => ScalarBinaryOp::Divide  (OpDataType::F32),
                    OPCODE_VECTOR_CW_MOD_F32 => ScalarBinaryOp::Modulo  (OpDataType::F32),
                    OPCODE_VECTOR_CW_ADD_I32 => ScalarBinaryOp::Add     (OpDataType::I32),
                    OPCODE_VECTOR_CW_SUB_I32 => ScalarBinaryOp::Subtract(OpDataType::I32),
                    OPCODE_VECTOR_CW_MUL_I32 => ScalarBinaryOp::Multiply(OpDataType::I32),
                    OPCODE_VECTOR_CW_DIV_I32 => ScalarBinaryOp::Divide  (OpDataType::I32),
                    OPCODE_VECTOR_CW_MOD_I32 => ScalarBinaryOp::Modulo  (OpDataType::I32),
                    OPCODE_VECTOR_CW_ATAN2   => ScalarBinaryOp::Atan2                    ,
                    OPCODE_VECTOR_CW_AND     => ScalarBinaryOp::And                      ,
                    OPCODE_VECTOR_CW_AND_NOT => ScalarBinaryOp::AndNot                   ,
                    OPCODE_VECTOR_CW_OR      => ScalarBinaryOp::Or                       ,
                    OPCODE_VECTOR_CW_XOR     => ScalarBinaryOp::Xor                      ,
                    _ => unreachable!()
                };
                module.instruction_buffer[instruction] = ShaderInstruction::VectorComponentwiseScalarBinaryOp {
                    src_a,
                    src_b,
                    dst,
                    op
                };
            },

            OPCODE_SCALAR_CONV_F32_TO_I32 |
            OPCODE_SCALAR_CONV_F32_TO_U32 |
            OPCODE_SCALAR_CONV_I32_TO_F32 |
            OPCODE_SCALAR_CONV_U32_TO_F32 |
            OPCODE_SCALAR_NEG_I32         |
            OPCODE_SCALAR_NEG_F32         |
            OPCODE_SCALAR_SIGN_I32        |
            OPCODE_SCALAR_SIGN_F32        |
            OPCODE_SCALAR_RECIP           |
            OPCODE_SCALAR_SIN             |
            OPCODE_SCALAR_COS             |
            OPCODE_SCALAR_TAN             |
            OPCODE_SCALAR_ASIN            |
            OPCODE_SCALAR_ACOS            |
            OPCODE_SCALAR_ATAN            |
            OPCODE_SCALAR_LN              |
            OPCODE_SCALAR_EXP             => {
                let dst = get_register::<Scalar>(&mut i, code)?;
                let src = get_register::<Scalar>(&mut i, code)?;
                let op = match opcode {
                    OPCODE_SCALAR_CONV_F32_TO_I32 => ScalarUnaryOp::Convert(OpDataTypeConversion::F32ToI32),
                    OPCODE_SCALAR_CONV_F32_TO_U32 => ScalarUnaryOp::Convert(OpDataTypeConversion::F32ToU32),
                    OPCODE_SCALAR_CONV_I32_TO_F32 => ScalarUnaryOp::Convert(OpDataTypeConversion::I32ToF32),
                    OPCODE_SCALAR_CONV_U32_TO_F32 => ScalarUnaryOp::Convert(OpDataTypeConversion::U32ToF32),
                    OPCODE_SCALAR_NEG_I32         => ScalarUnaryOp::Negative (OpDataType::I32),
                    OPCODE_SCALAR_NEG_F32         => ScalarUnaryOp::Negative (OpDataType::F32),
                    OPCODE_SCALAR_SIGN_I32        => ScalarUnaryOp::Sign     (OpDataType::I32),
                    OPCODE_SCALAR_SIGN_F32        => ScalarUnaryOp::Sign     (OpDataType::F32),
                    OPCODE_SCALAR_RECIP           => ScalarUnaryOp::Reciporocal,
                    OPCODE_SCALAR_SIN             => ScalarUnaryOp::Sin,
                    OPCODE_SCALAR_COS             => ScalarUnaryOp::Cos,
                    OPCODE_SCALAR_TAN             => ScalarUnaryOp::Tan,
                    OPCODE_SCALAR_ASIN            => ScalarUnaryOp::ASin,
                    OPCODE_SCALAR_ACOS            => ScalarUnaryOp::ACos,
                    OPCODE_SCALAR_ATAN            => ScalarUnaryOp::Atan,
                    OPCODE_SCALAR_LN              => ScalarUnaryOp::Ln,
                    OPCODE_SCALAR_EXP             => ScalarUnaryOp::Exp,
                    _ => unreachable!()
                };
                module.instruction_buffer[instruction] = ShaderInstruction::ScalarUnaryOp {
                    src,
                    dst,
                    op
                };
            },

            OPCODE_VECTOR_CW_CONV_F32_TO_I32 |
            OPCODE_VECTOR_CW_CONV_F32_TO_U32 |
            OPCODE_VECTOR_CW_CONV_I32_TO_F32 |
            OPCODE_VECTOR_CW_CONV_U32_TO_F32 |
            OPCODE_VECTOR_CW_NEG_I32         |
            OPCODE_VECTOR_CW_NEG_F32         |
            OPCODE_VECTOR_CW_SIGN_I32        |
            OPCODE_VECTOR_CW_SIGN_F32        |
            OPCODE_VECTOR_CW_RECIP           |
            OPCODE_VECTOR_CW_SIN             |
            OPCODE_VECTOR_CW_COS             |
            OPCODE_VECTOR_CW_TAN             |
            OPCODE_VECTOR_CW_ASIN            |
            OPCODE_VECTOR_CW_ACOS            |
            OPCODE_VECTOR_CW_ATAN            |
            OPCODE_VECTOR_CW_LN              |
            OPCODE_VECTOR_CW_EXP             => {
                let dst = get_register::<Vector>(&mut i, code)?;
                let src = get_register::<Vector>(&mut i, code)?;
                let op = match opcode {
                    OPCODE_VECTOR_CW_CONV_F32_TO_I32 => ScalarUnaryOp::Convert(OpDataTypeConversion::F32ToI32),
                    OPCODE_VECTOR_CW_CONV_F32_TO_U32 => ScalarUnaryOp::Convert(OpDataTypeConversion::F32ToU32),
                    OPCODE_VECTOR_CW_CONV_I32_TO_F32 => ScalarUnaryOp::Convert(OpDataTypeConversion::I32ToF32),
                    OPCODE_VECTOR_CW_CONV_U32_TO_F32 => ScalarUnaryOp::Convert(OpDataTypeConversion::U32ToF32),
                    OPCODE_VECTOR_CW_NEG_I32         => ScalarUnaryOp::Negative (OpDataType::I32),
                    OPCODE_VECTOR_CW_NEG_F32         => ScalarUnaryOp::Negative (OpDataType::F32),
                    OPCODE_VECTOR_CW_SIGN_I32        => ScalarUnaryOp::Sign     (OpDataType::I32),
                    OPCODE_VECTOR_CW_SIGN_F32        => ScalarUnaryOp::Sign     (OpDataType::F32),
                    OPCODE_VECTOR_CW_RECIP           => ScalarUnaryOp::Reciporocal,
                    OPCODE_VECTOR_CW_SIN             => ScalarUnaryOp::Sin,
                    OPCODE_VECTOR_CW_COS             => ScalarUnaryOp::Cos,
                    OPCODE_VECTOR_CW_TAN             => ScalarUnaryOp::Tan,
                    OPCODE_VECTOR_CW_ASIN            => ScalarUnaryOp::ASin,
                    OPCODE_VECTOR_CW_ACOS            => ScalarUnaryOp::ACos,
                    OPCODE_VECTOR_CW_ATAN            => ScalarUnaryOp::Atan,
                    OPCODE_VECTOR_CW_LN              => ScalarUnaryOp::Ln,
                    OPCODE_VECTOR_CW_EXP             => ScalarUnaryOp::Exp,
                    _ => unreachable!()
                };
                module.instruction_buffer[instruction] = ShaderInstruction::VectorComponentwiseScalarUnaryOp {
                    src,
                    dst,
                    op
                };
            },

            OPCODE_MAG2 |
            OPCODE_MAG3 |
            OPCODE_MAG4 |
            OPCODE_SQ_MAG2 |
            OPCODE_SQ_MAG3 |
            OPCODE_SQ_MAG4 => {
                let dst = get_register::<Scalar>(&mut i, code)?;
                let src = get_register::<Vector>(&mut i, code)?;
                let op = match opcode {
                    OPCODE_MAG2    => VectorToScalarUnaryOp::Magnitude2,
                    OPCODE_MAG3    => VectorToScalarUnaryOp::Magnitude3,
                    OPCODE_MAG4    => VectorToScalarUnaryOp::Magnitude4,
                    OPCODE_SQ_MAG2 => VectorToScalarUnaryOp::SquareMagnitude2,
                    OPCODE_SQ_MAG3 => VectorToScalarUnaryOp::SquareMagnitude3,
                    OPCODE_SQ_MAG4 => VectorToScalarUnaryOp::SquareMagnitude4,
                    _ => unreachable!()
                };
                module.instruction_buffer[instruction] = ShaderInstruction::VectorToScalarUnaryOp {
                    src,
                    dst,
                    op
                };
            },

            OPCODE_NORM2 |
            OPCODE_NORM3 |
            OPCODE_NORM4 => {
                let dst = get_register::<Vector>(&mut i, code)?;
                let src = get_register::<Vector>(&mut i, code)?;
                let op = match opcode {
                    OPCODE_NORM2 => VectorToVectorUnaryOp::Normalize2,
                    OPCODE_NORM3 => VectorToVectorUnaryOp::Normalize3,
                    OPCODE_NORM4 => VectorToVectorUnaryOp::Normalize4,
                    _ => unreachable!()
                };
                module.instruction_buffer[instruction] = ShaderInstruction::VectorToVectorUnaryOp {
                    src,
                    dst,
                    op
                };
            },

            _ => Err(ShaderParseError::UnknownOpcode)?
        }
        instruction += 1;
    }
    module.instruction_count = instruction;
    module.shader_type = shader_type;
    Ok(())
}