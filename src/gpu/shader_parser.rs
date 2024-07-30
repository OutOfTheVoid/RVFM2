use std::marker::PhantomData;

use super::shader::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Compute
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShaderParseError {
    UnexpectedEndOfCode,
    InvalidRegisterAddress,
    UnknownOpcode
}

const SHADER_OPCODE_VECTOR_PUSH : u8 = 0x00;
const SHADER_OPCODE_SCALAR_PUSH : u8 = 0x01;
const SHADER_OPCODE_VECTOR_POP  : u8 = 0x02;
const SHADER_OPCODE_SCALAR_POP  : u8 = 0x03;

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
    let t = get_byte(i, code)?;
    let index = get_byte(i, code)?;
    Ok(match t {
        REGTYPE_LOCAL    => RegisterAddress::Local   (index, PhantomData),
        REGTYPE_INPUT    => RegisterAddress::Input   (index, PhantomData),
        REGTYPE_OUTPUT   => RegisterAddress::Output  (index, PhantomData),
        REGTYPE_CONSTANT => RegisterAddress::Constant(index, PhantomData),
        _ => Err(ShaderParseError::InvalidRegisterAddress)?
    })
}

pub fn parse_shader_bytecode(shader_type: ShaderType, code: &[u8]) -> Result<Box<[ShaderInstruction]>, ShaderParseError> {
    let mut instructions = Vec::new();
    let mut i = 0;
    while i < code.len() {
        match code[i] {
            SHADER_OPCODE_VECTOR_PUSH => {
                let src = get_register::<Vector>(&mut i, code)?;
                instructions.push(ShaderInstruction::PushVector(src))
            },
            SHADER_OPCODE_SCALAR_PUSH => {
                let src = get_register::<Scalar>(&mut i, code)?;
                instructions.push(ShaderInstruction::PushScalar(src))
            },
            SHADER_OPCODE_VECTOR_POP => {
                let dst = get_register::<Vector>(&mut i, code)?;
                instructions.push(ShaderInstruction::PopVector(dst))
            },
            SHADER_OPCODE_SCALAR_POP => {
                let dst = get_register::<Scalar>(&mut i, code)?;
                instructions.push(ShaderInstruction::PopScalar(dst))
            },
            _ => Err(ShaderParseError::UnknownOpcode)?
        }
    }
    Ok(instructions.into_boxed_slice())
}