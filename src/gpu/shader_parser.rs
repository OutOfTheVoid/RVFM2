use std::marker::PhantomData;

use super::shader::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShaderParseError {
    UnexpectedEndOfCode,
    InvalidRegisterAddress(usize),
    UnknownOpcode,
    ShaderTooLong,
}

const OPCODE_VECTOR_PUSH                          : u8 = 0x00;
const OPCODE_SCALAR_PUSH                          : u8 = 0x01;
const OPCODE_VECTOR_POP                           : u8 = 0x02;
const OPCODE_SCALAR_POP                           : u8 = 0x03;
const OPCODE_VECTOR_COPY                          : u8 = 0x04;
const OPCODE_SCALAR_COPY                          : u8 = 0x05;
const OPCODE_VECTOR_COMPONENT_TO_SCALAR_COPY      : u8 = 0x06;
const OPCODE_SCALAR_TO_VECTOR_COMPONENT_COPY      : u8 = 0x07;
const OPCODE_COND_VECTOR_COPY                     : u8 = 0x04;
const OPCODE_COND_SCALAR_COPY                     : u8 = 0x05;
const OPCODE_COND_VECTOR_COMPONENT_TO_SCALAR_COPY : u8 = 0x06;
const OPCODE_COND_SCALAR_TO_VECTOR_COMPONENT_COPY : u8 = 0x07;

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
            _ => Err(ShaderParseError::UnknownOpcode)?
        }
        instruction += 1;
    }
    module.instruction_count = instruction;
    Ok(())
}