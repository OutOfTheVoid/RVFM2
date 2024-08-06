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
            OPCODE_SCALAR_MOD_I32 => {
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
            OPCODE_VECTOR_CW_MOD_I32 => {
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
                    _ => unreachable!()
                };
                module.instruction_buffer[instruction] = ShaderInstruction::VectorComponentwiseScalarBinaryOp {
                    src_a,
                    src_b,
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