use std::marker::PhantomData;
use std::ptr::read;
use super::types::PixelDataType;
use super::buffer::BufferModule;
use super::texture::TextureModule;

pub trait RegisterType {}


#[derive(Eq, Copy, Clone, Debug)]
pub struct Vector;
impl RegisterType for Vector {}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

#[derive(Eq, Copy, Clone, Debug)]
pub struct Scalar;
impl RegisterType for Scalar {}

impl PartialEq for Scalar {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RegisterAddress<Type: RegisterType> {
    Local(u16, PhantomData<Type>),
    Constant(u8, PhantomData<Type>),
    Input(u8, PhantomData<Type>),
    Output(u8, PhantomData<Type>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VectorChannel {
    X = 0,
    Y = 1,
    Z = 2,
    W = 3,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BufferReadType {
    D8,
    D16,
    D32
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VectorBufferReadType {
    Scalar(BufferReadType),
    V2(BufferReadType),
    V3(BufferReadType),
    V4(BufferReadType)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BufferWriteType {
    I8,
    I16,
    I32,
    U8,
    U16,
    U32,
    INorm8,
    INorm16,
    INorm32,
    UNorm8,
    UNorm16,
    UNorm32,
    F32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VectorBufferWriteType {
    Scalar(BufferWriteType),
    V2(BufferWriteType),
    V3(BufferWriteType),
    V4(BufferWriteType)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OpDataType {
    F32,
    I32
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OpDataTypeConversion {
    F32ToI32,
    F32ToU32,
    I32toF32,
    U32ToF32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScalarUnaryOp {
    Convert     ( OpDataTypeConversion ),
    Negative    ( OpDataType           ),
    Reciporocal ( OpDataType           ),
    Sign        ( OpDataType           ),

    // F32 only
    Sin  ,
    Cos  ,
    Tan  ,
    Atan ,
    Log  ,
    Exp  ,

}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScalarBinaryOp {
    Add      ( OpDataType ),
    Subtract ( OpDataType ),
    Multiply ( OpDataType ),
    Divide   ( OpDataType ),
    Modulo   ( OpDataType ),
    Atan2    ( OpDataType ),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScalarTernaryOp {
    Fma      ( OpDataType ),
    Lerp     ( OpDataType ),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VectorToVectorUnaryOp {
    Normalize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VectorToScalarUnaryOp {
    Magnitude,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VectorToVectorBinaryOp {
    CrossProduct3,
}

const COMPONENT_MASK_X: u8 = 1;
const COMPONENT_MASK_Y: u8 = 2;
const COMPONENT_MASK_Z: u8 = 4;
const COMPONENT_MASK_W: u8 = 8;

pub enum ShaderInstruction {
    PushVector(RegisterAddress<Vector>),
    PushScalar(RegisterAddress<Scalar>),
    PopVector(RegisterAddress<Vector>),
    PopScalar(RegisterAddress<Scalar>),
    CopyVectorRegister {
        from: RegisterAddress<Vector>,
        to: RegisterAddress<Vector>
    },
    CopyScalarRegister {
        from: RegisterAddress<Scalar>,
        to: RegisterAddress<Scalar>
    },
    CopyVectorComponentToScalar {
        channel: VectorChannel,
        from: RegisterAddress<Vector>,
        to: RegisterAddress<Scalar>
    },
    CopyScalarToVectorComponent {
        channel: VectorChannel,
        from: RegisterAddress<Scalar>,
        to: RegisterAddress<Vector>
    },
    CopyScalarToVectorComponentsMasked {
        channel: VectorChannel,
        from: RegisterAddress<Scalar>,
        to: RegisterAddress<Vector>,
        mask: u8
    },
    ReadBufferToVector {
        data_type: VectorBufferReadType,
        vector: RegisterAddress<Vector>,
        offset: u32,
        addr_src_u32: Option<RegisterAddress<Scalar>>,
        buffer: u8,
    },
    WriteVectorToBuffer {
        data_type: VectorBufferWriteType,
        src: RegisterAddress<Vector>,
        offset: u32,
        addr_src_u32: Option<RegisterAddress<Scalar>>,
        buffer: u8,
    },
    ReadBufferToScalar {
        data_type: BufferReadType,
        scalar: RegisterAddress<Scalar>,
        offset: u32,
        addr_src_u32: Option<RegisterAddress<Scalar>>,
        buffer: u8,
    },
    WriteScalarToBuffer {
        data_type: BufferWriteType,
        scalar: RegisterAddress<Scalar>,
        offset: u32,
        addr_dst_u32: Option<RegisterAddress<Scalar>>,
        buffer: u8,
    },
    LoadTextureVector {
        src_xy_u32: RegisterAddress<Vector>,
        dst: RegisterAddress<Vector>,
        texture: u8,
    },
    LoadTextureScalar {
        src_xy_u32: RegisterAddress<Vector>,
        channel: VectorChannel,
        dst: RegisterAddress<Scalar>,
        texture: u8,
    },
    ScalarUnaryOp {
        src: RegisterAddress<Scalar>,
        dst: RegisterAddress<Scalar>,
        op: ScalarUnaryOp
    },
    ScalarBinaryOp {
        src_a: RegisterAddress<Scalar>,
        src_b: RegisterAddress<Scalar>,
        dst: RegisterAddress<Scalar>,
        op: ScalarBinaryOp,
    },
    VectorComponentwiseScalarUnaryOp {
        src: RegisterAddress<Vector>,
        dst: RegisterAddress<Vector>,
        op: ScalarUnaryOp,
    },
    VectorComponentwiseScalarBinaryOp {
        src_a: RegisterAddress<Vector>,
        src_b: RegisterAddress<Vector>,
        dst: RegisterAddress<Vector>,
        op: ScalarBinaryOp,
    },
    VectorComponentwiseScalarTernaryOp {
        src_a: RegisterAddress<Vector>,
        src_b: RegisterAddress<Vector>,
        src_c: RegisterAddress<Vector>,
        dst: RegisterAddress<Vector>,
        op: ScalarTernaryOp,
    },
    VectorComponentwiseScalarUnaryOpMasked {
        src: RegisterAddress<Vector>,
        dst: RegisterAddress<Vector>,
        op: ScalarUnaryOp,
        mask: u8
    },
    VectorComponentwiseScalarBinaryOpMasked {
        src_a: RegisterAddress<Vector>,
        src_b: RegisterAddress<Vector>,
        dst: RegisterAddress<Vector>,
        op: ScalarBinaryOp,
        mask: u8,
    },
    VectorComponentwiseScalarTernaryOpMasked {
        src_a: RegisterAddress<Vector>,
        src_b: RegisterAddress<Vector>,
        src_c: RegisterAddress<Vector>,
        dst: RegisterAddress<Vector>,
        op: ScalarTernaryOp,
        mask: u8,
    },
    VectorToVectorUnaryOp {
        src: RegisterAddress<Vector>,
        dst: RegisterAddress<Vector>,
        op: VectorToVectorUnaryOp
    },
    VectorToScalarUnaryOp {
        src: RegisterAddress<Vector>,
        dst: RegisterAddress<Scalar>,
        op: VectorToScalarUnaryOp
    },
}

#[derive(Debug)]
pub struct ShadingUnitRunContext<'i, 'o> {
    pub scalar_input_array: InputArray<'i, [u32; CORE_COUNT]>,
    pub vector_input_array: InputArray<'i, [[u32; 4]; CORE_COUNT]>,
    pub scalar_constant_array: InputArray<'i, u32>,
    pub vector_constant_array: InputArray<'i, [u32; 4]>,
    pub scalar_output_array: OutputArray<'o, [u32; CORE_COUNT]>,
    pub vector_output_array: OutputArray<'o, [[u32; 4]; CORE_COUNT]>,
}

const CORE_COUNT: usize = 0x1000;
const STACK_SIZE: usize = 0x400;
const LOCAL_COUNT: usize = 0x20;
const INPUT_COUNT: usize = 0x100;
const OUTPUT_COUNT: usize = 0x100;

type InputArray<'a, T> = &'a [T; INPUT_COUNT];
type OutputArray<'a, T> = &'a mut [T; OUTPUT_COUNT];

struct ShadingUnitContext {
    scalar_stack_array: [[ u32     ; CORE_COUNT]; STACK_SIZE],
    scalar_stack_size:  usize,
    vector_stack_array: [[[u32; 4] ; CORE_COUNT]; STACK_SIZE],
    vector_stack_size:  usize,
    scalar_locals:      [[ u32     ; CORE_COUNT]; LOCAL_COUNT],
    vector_locals:      [[[u32; 4] ; CORE_COUNT]; LOCAL_COUNT],
}

enum RegisterRead<'a, T> {
    Core(&'a [T; CORE_COUNT]),
    Uniform(&'a T),
}

impl<'a, T> RegisterRead<'a, T> {
    fn to_ptr(self) -> RegisterReadPtr<T> {
        match self {
            Self::Core(core_ref) => RegisterReadPtr::Core(core_ref as *const _),
            Self::Uniform(uniform_ref) => RegisterReadPtr::Uniform(uniform_ref as *const _),
        }
    }
}

enum RegisterReadPtr<T> {
    Core(*const [T; CORE_COUNT]),
    Uniform(*const T),
}

impl<T> RegisterReadPtr<T> {
    fn to_ref<'a>(self) -> RegisterRead<'a, T> {
        match self {
            Self::Core(core_ptr) => RegisterRead::Core(unsafe { &*core_ptr }),
            Self::Uniform(uniform_ptr) => RegisterRead::Uniform(unsafe { &*uniform_ptr }),
        }
    }
}

impl ShadingUnitContext {
    pub fn run_instruction<'i, 'o, 'rc>(&mut self, instruction: &ShaderInstruction, run_context: &'rc mut ShadingUnitRunContext<'i, 'o>, buffer_modules: &mut [BufferModule; 256], texture_modules: &mut [TextureModule; 64]) -> Option<()> {
        match instruction {
            ShaderInstruction::PushVector(register) => {
                let source_register = self.read_vector_register(*register, run_context);
                let stack_slot = &mut self.vector_stack_array[self.vector_stack_size];
                match source_register {
                    RegisterRead::Core(register_list) => stack_slot.copy_from_slice(register_list),
                    RegisterRead::Uniform(register) => stack_slot.fill(*register),
                }
                self.vector_stack_size += 1;
            },
            ShaderInstruction::PushScalar(register) => {
                let source_register = self.read_scalar_register(*register, run_context);
                let stack_slot = &mut self.scalar_stack_array[self.vector_stack_size];
                match source_register {
                    RegisterRead::Core(register_list) => stack_slot.copy_from_slice(register_list),
                    RegisterRead::Uniform(register) => stack_slot.fill(*register),
                }
                self.scalar_stack_size += 1;
            },
            ShaderInstruction::PopVector(register) => {
                let register_list = self.write_vector_register(*register, run_context)?;
                self.vector_stack_size -= 1;
                let stack_slot = &self.vector_stack_array[self.vector_stack_size];
                register_list.copy_from_slice(stack_slot);
            },
            ShaderInstruction::PopScalar(register) => {
                let register_list = self.write_scalar_register(*register, run_context)?;
                self.scalar_stack_size -= 1;
                let stack_slot = &self.scalar_stack_array[self.scalar_stack_size];
                register_list.copy_from_slice(stack_slot);
            },
            ShaderInstruction::CopyVectorRegister { from, to } => {
                if *from == *to { None? }
                let from_register = self.read_vector_register(*from, run_context);
                let to_register_list = self.write_vector_register(*to, run_context)?;
                match from_register {
                    RegisterRead::Core(register_list) => todo!(),
                    RegisterRead::Uniform(register) => todo!(),
                }
            },
            ShaderInstruction::CopyScalarRegister { from, to } => {
                if *from == *to { None? }
                let from_register = self.read_scalar_register(*from, run_context);
                let to_register_list = self.write_scalar_register(*to, run_context)?;
                match from_register {
                    RegisterRead::Core(register_list) => todo!(),
                    RegisterRead::Uniform(register) => todo!(),
                }
            },
            ShaderInstruction::CopyVectorComponentToScalar { channel, from, to } => {
                let from_register = self.read_vector_register(*from, run_context);
                let to_register_list = self.write_scalar_register(*to, run_context)?;
                match from_register {
                    RegisterRead::Core(from_register_list) =>
                        (0..CORE_COUNT)
                            .for_each(|i| to_register_list[i] = from_register_list[i][*channel as usize]),
                    RegisterRead::Uniform(from_register) =>
                        (0..CORE_COUNT)
                            .for_each(|i| to_register_list[i] = from_register[*channel as usize]),
                }
            },
            ShaderInstruction::CopyScalarToVectorComponent { channel, from, to } => {
                let from_register = self.read_scalar_register(*from, run_context);
                let to_register_list = self.write_vector_register(*to, run_context)?;
                match from_register {
                    RegisterRead::Core(from_register_list) =>
                        (0..CORE_COUNT)
                            .for_each(|i| to_register_list[i][*channel as usize] = from_register_list[i]),
                    RegisterRead::Uniform(from_register) =>
                        (0..CORE_COUNT)
                            .for_each(|i| to_register_list[i][*channel as usize] = *from_register),
                }
            },
            ShaderInstruction::CopyScalarToVectorComponentsMasked { channel, from, to, mask } => {
                let from_register = self.read_scalar_register(*from, run_context);
                let to_register_list = self.write_vector_register(*to, run_context)?;
                match from_register {
                    RegisterRead::Core(from_register_list) =>
                        (0..4).for_each(|c| {
                            (0..CORE_COUNT).for_each(|i| {
                                if (mask & (1 << c)) != 0 {
                                    to_register_list[i][c] = from_register_list[i];
                                }
                            })
                        }),
                    RegisterRead::Uniform(from_register) =>
                        (0..4).for_each(|c| {
                            (0..CORE_COUNT).for_each(|i| {
                                to_register_list[i][c] = *from_register;
                            })
                        }),
                }
            },
            ShaderInstruction::ReadBufferToVector { data_type, vector, offset, addr_src_u32, buffer } => {
                let buffer = &buffer_modules[*buffer as usize];
                let buffer_bytes = buffer.bytes();
                let vector_register = self.write_vector_register(*vector, run_context)?;
                let read_fn = match data_type {
                    VectorBufferReadType::Scalar(BufferReadType::D8) => |bytes: &[u8], offset: usize|
                        [
                            read_bytes_u8(bytes, offset) as u32,
                            0,
                            0,
                            0
                        ],
                    VectorBufferReadType::Scalar(BufferReadType::D16) => |bytes: &[u8], offset: usize| 
                        [
                            read_bytes_u16(bytes, offset) as u32,
                            0,
                            0,
                            0
                        ],
                    VectorBufferReadType::Scalar(BufferReadType::D32) => |bytes: &[u8], offset: usize| 
                        [
                            read_bytes_u32(bytes, offset + 0),
                            0,
                            0,
                            0
                        ],
                    VectorBufferReadType::V2(BufferReadType::D8) => |bytes: &[u8], offset: usize|
                        [
                            read_bytes_u8(bytes, offset + 0) as u32,
                            read_bytes_u8(bytes, offset + 1) as u32,
                            0,
                            0
                        ],
                    VectorBufferReadType::V2(BufferReadType::D16) => |bytes: &[u8], offset: usize| 
                        [
                            read_bytes_u16(bytes, offset + 0) as u32,
                            read_bytes_u16(bytes, offset + 2) as u32,
                            0,
                            0
                        ],
                    VectorBufferReadType::V2(BufferReadType::D32) => |bytes: &[u8], offset: usize| 
                        [
                            read_bytes_u32(bytes, offset + 0),
                            read_bytes_u32(bytes, offset + 4),
                            0,
                            0
                        ],
                    VectorBufferReadType::V3(BufferReadType::D8) => |bytes: &[u8], offset: usize|
                        [
                            read_bytes_u8(bytes, offset + 0) as u32,
                            read_bytes_u8(bytes, offset + 1) as u32,
                            read_bytes_u8(bytes, offset + 2) as u32,
                            0
                        ],
                    VectorBufferReadType::V3(BufferReadType::D16) => |bytes: &[u8], offset: usize| 
                        [
                            read_bytes_u16(bytes, offset + 0) as u32,
                            read_bytes_u16(bytes, offset + 2) as u32,
                            read_bytes_u16(bytes, offset + 4) as u32,
                            0
                        ],
                    VectorBufferReadType::V3(BufferReadType::D32) => |bytes: &[u8], offset: usize| 
                        [
                            read_bytes_u32(bytes, offset + 0),
                            read_bytes_u32(bytes, offset + 4),
                            read_bytes_u32(bytes, offset + 8),
                            0
                        ],
                    VectorBufferReadType::V4(BufferReadType::D8) => |bytes: &[u8], offset: usize|
                        [
                            read_bytes_u8(bytes, offset + 0) as u32,
                            read_bytes_u8(bytes, offset + 1) as u32,
                            read_bytes_u8(bytes, offset + 2) as u32,
                            read_bytes_u8(bytes, offset + 3) as u32,
                        ],
                    VectorBufferReadType::V4(BufferReadType::D16) => |bytes: &[u8], offset: usize| 
                        [
                            read_bytes_u16(bytes, offset + 0) as u32,
                            read_bytes_u16(bytes, offset + 2) as u32,
                            read_bytes_u16(bytes, offset + 4) as u32,
                            read_bytes_u16(bytes, offset + 6) as u32,
                        ],
                    VectorBufferReadType::V4(BufferReadType::D32) => |bytes: &[u8], offset: usize| 
                        [
                            read_bytes_u32(bytes, offset + 0),
                            read_bytes_u32(bytes, offset + 4),
                            read_bytes_u32(bytes, offset + 8),
                            read_bytes_u32(bytes, offset + 12),
                        ],
                };
                match addr_src_u32 {
                    Some(src_register_addr) => {
                        match self.read_scalar_register(*src_register_addr, run_context) {
                            RegisterRead::Core(addr_register_list) => {
                                (0..CORE_COUNT).for_each(|i| {
                                    vector_register[i] = read_fn(buffer_bytes, *offset as usize + addr_register_list[i] as usize);
                                })
                            },
                            RegisterRead::Uniform(addr_register) => {
                                let value = read_fn(buffer_bytes, *offset as usize + *addr_register as usize);
                                vector_register.fill(value);
                            }
                        }
                    },
                    None => {
                        let value = read_fn(buffer_bytes, *offset as usize);
                        vector_register.fill(value);
                    }
                }
            },
            ShaderInstruction::WriteVectorToBuffer{ data_type, src, offset, addr_src_u32, buffer } => {
                let buffer = &mut buffer_modules[*buffer as usize];
                let bytes = buffer.bytes_mut();
                let vector_register = self.read_vector_register(*src, run_context);
                let write_fn = match *data_type {
                    VectorBufferWriteType::Scalar(BufferWriteType::I8 ) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u8 (&[vector[0] as  i8 as  u8], bytes, *offset as usize),
                    VectorBufferWriteType::Scalar(BufferWriteType::I16) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u16(&[vector[0] as i16 as u16], bytes, *offset as usize),
                    VectorBufferWriteType::Scalar(BufferWriteType::I32) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u32(&[vector[0] as i32 as u32], bytes, *offset as usize),
                    VectorBufferWriteType::Scalar(BufferWriteType::U8 ) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u8 (&[vector[0] as  u8], bytes, *offset as usize),
                    VectorBufferWriteType::Scalar(BufferWriteType::U16) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u16(&[vector[0] as u16], bytes, *offset as usize),
                    VectorBufferWriteType::Scalar(BufferWriteType::U32) |
                    VectorBufferWriteType::Scalar(BufferWriteType::F32) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u32(&[vector[0]], bytes, *offset as usize),
                    VectorBufferWriteType::Scalar(BufferWriteType::INorm8) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u8(&[f32_bits_to_inorm8_bits(vector[0])], bytes, *offset as usize),
                    VectorBufferWriteType::Scalar(BufferWriteType::INorm16) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u16(&[f32_bits_to_inorm16_bits(vector[0])], bytes, *offset as usize),
                    VectorBufferWriteType::Scalar(BufferWriteType::INorm32) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u32(&[f32_bits_to_inorm32_bits(vector[0])], bytes, *offset as usize),
                    VectorBufferWriteType::Scalar(BufferWriteType::UNorm8) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u8(&[f32_bits_to_unorm8_bits(vector[0])], bytes, *offset as usize),
                    VectorBufferWriteType::Scalar(BufferWriteType::UNorm16) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u16(&[f32_bits_to_unorm16_bits(vector[0])], bytes, *offset as usize),
                    VectorBufferWriteType::Scalar(BufferWriteType::UNorm32) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u32(&[f32_bits_to_unorm32_bits(vector[0])], bytes, *offset as usize),
                    
                    VectorBufferWriteType::V2(BufferWriteType::I8 ) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u8 (&[vector[0] as  i8 as  u8, vector[1] as  i8 as  u8], bytes, *offset as usize),
                    VectorBufferWriteType::V2(BufferWriteType::I16) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u16(&[vector[0] as i16 as u16, vector[1] as i16 as u16], bytes, *offset as usize),
                    VectorBufferWriteType::V2(BufferWriteType::I32) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u32(&[vector[0] as i32 as u32, vector[1] as i32 as u32], bytes, *offset as usize),
                    VectorBufferWriteType::V2(BufferWriteType::U8 ) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u8 (&[vector[0] as  u8, vector[1] as  u8], bytes, *offset as usize),
                    VectorBufferWriteType::V2(BufferWriteType::U16) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u16(&[vector[0] as u16, vector[1] as u16], bytes, *offset as usize),
                    VectorBufferWriteType::V2(BufferWriteType::U32) |
                    VectorBufferWriteType::V2(BufferWriteType::F32) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u32(&[vector[0], vector[1]], bytes, *offset as usize),
                    VectorBufferWriteType::V2(BufferWriteType::INorm8) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u8(&[f32_bits_to_inorm8_bits(vector[0]), f32_bits_to_inorm8_bits(vector[1])], bytes, *offset as usize),
                    VectorBufferWriteType::V2(BufferWriteType::INorm16) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u16(&[f32_bits_to_inorm16_bits(vector[0]), f32_bits_to_inorm16_bits(vector[1])], bytes, *offset as usize),
                    VectorBufferWriteType::V2(BufferWriteType::INorm32) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u32(&[f32_bits_to_inorm32_bits(vector[0]), f32_bits_to_inorm32_bits(vector[1])], bytes, *offset as usize),
                    VectorBufferWriteType::V2(BufferWriteType::UNorm8) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u8(&[f32_bits_to_unorm8_bits(vector[0]), f32_bits_to_unorm8_bits(vector[1])], bytes, *offset as usize),
                    VectorBufferWriteType::V2(BufferWriteType::UNorm16) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u16(&[f32_bits_to_unorm16_bits(vector[0]), f32_bits_to_unorm16_bits(vector[1])], bytes, *offset as usize),
                    VectorBufferWriteType::V2(BufferWriteType::UNorm32) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u32(&[f32_bits_to_unorm32_bits(vector[0]), f32_bits_to_unorm32_bits(vector[1])], bytes, *offset as usize),

                    VectorBufferWriteType::V3(BufferWriteType::I8 ) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u8 (&[vector[0] as  i8 as  u8, vector[1] as  i8 as  u8, vector[2] as  i8 as  u8], bytes, *offset as usize),
                    VectorBufferWriteType::V3(BufferWriteType::I16) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u16(&[vector[0] as i16 as u16, vector[1] as i16 as u16, vector[2] as i16 as u16], bytes, *offset as usize),
                    VectorBufferWriteType::V3(BufferWriteType::I32) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u32(&[vector[0] as i32 as u32, vector[1] as i32 as u32, vector[2] as i32 as u32], bytes, *offset as usize),
                    VectorBufferWriteType::V3(BufferWriteType::U8 ) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u8 (&[vector[0] as  u8, vector[1] as  u8, vector[2] as  u8], bytes, *offset as usize),
                    VectorBufferWriteType::V3(BufferWriteType::U16) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u16(&[vector[0] as u16, vector[1] as u16, vector[2] as u16], bytes, *offset as usize),
                    VectorBufferWriteType::V3(BufferWriteType::U32) |
                    VectorBufferWriteType::V3(BufferWriteType::F32) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u32(&[vector[0], vector[1], vector[2]], bytes, *offset as usize),
                    VectorBufferWriteType::V3(BufferWriteType::INorm8) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u8(&[f32_bits_to_inorm8_bits(vector[0]), f32_bits_to_inorm8_bits(vector[1]), f32_bits_to_inorm8_bits(vector[2])], bytes, *offset as usize),
                    VectorBufferWriteType::V3(BufferWriteType::INorm16) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u16(&[f32_bits_to_inorm16_bits(vector[0]), f32_bits_to_inorm16_bits(vector[1]), f32_bits_to_inorm16_bits(vector[2])], bytes, *offset as usize),
                    VectorBufferWriteType::V3(BufferWriteType::INorm32) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u32(&[f32_bits_to_inorm32_bits(vector[0]), f32_bits_to_inorm32_bits(vector[1]), f32_bits_to_inorm32_bits(vector[2])], bytes, *offset as usize),
                    VectorBufferWriteType::V3(BufferWriteType::UNorm8) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u8(&[f32_bits_to_unorm8_bits(vector[0]), f32_bits_to_unorm8_bits(vector[1]), f32_bits_to_unorm8_bits(vector[2])], bytes, *offset as usize),
                    VectorBufferWriteType::V3(BufferWriteType::UNorm16) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u16(&[f32_bits_to_unorm16_bits(vector[0]), f32_bits_to_unorm16_bits(vector[1]), f32_bits_to_unorm16_bits(vector[2])], bytes, *offset as usize),
                    VectorBufferWriteType::V3(BufferWriteType::UNorm32) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u32(&[f32_bits_to_unorm32_bits(vector[0]), f32_bits_to_unorm32_bits(vector[1]), f32_bits_to_unorm32_bits(vector[2])], bytes, *offset as usize),

                    VectorBufferWriteType::V4(BufferWriteType::I8 ) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u8 (&[vector[0] as  i8 as  u8, vector[1] as  i8 as  u8, vector[2] as  i8 as  u8, vector[3] as  i8 as  u8], bytes, *offset as usize),
                    VectorBufferWriteType::V4(BufferWriteType::I16) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u16(&[vector[0] as i16 as u16, vector[1] as i16 as u16, vector[2] as i16 as u16, vector[3] as i16 as u16], bytes, *offset as usize),
                    VectorBufferWriteType::V4(BufferWriteType::I32) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u32(&[vector[0] as i32 as u32, vector[1] as i32 as u32, vector[2] as i32 as u32, vector[3] as i32 as u32], bytes, *offset as usize),
                    VectorBufferWriteType::V4(BufferWriteType::U8 ) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u8 (&[vector[0] as  u8, vector[1] as  u8, vector[2] as  u8, vector[3] as  u8], bytes, *offset as usize),
                    VectorBufferWriteType::V4(BufferWriteType::U16) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u16(&[vector[0] as u16, vector[1] as u16, vector[2] as u16, vector[3] as u16], bytes, *offset as usize),
                    VectorBufferWriteType::V4(BufferWriteType::U32) |
                    VectorBufferWriteType::V4(BufferWriteType::F32) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u32(&[vector[0], vector[1], vector[2], vector[3]], bytes, *offset as usize),
                    VectorBufferWriteType::V4(BufferWriteType::INorm8) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u8(&[f32_bits_to_inorm8_bits(vector[0]), f32_bits_to_inorm8_bits(vector[1]), f32_bits_to_inorm8_bits(vector[2]), f32_bits_to_inorm8_bits(vector[3])], bytes, *offset as usize),
                    VectorBufferWriteType::V4(BufferWriteType::INorm16) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u16(&[f32_bits_to_inorm16_bits(vector[0]), f32_bits_to_inorm16_bits(vector[1]), f32_bits_to_inorm16_bits(vector[2]), f32_bits_to_inorm16_bits(vector[3])], bytes, *offset as usize),
                    VectorBufferWriteType::V4(BufferWriteType::INorm32) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u32(&[f32_bits_to_inorm32_bits(vector[0]), f32_bits_to_inorm32_bits(vector[1]), f32_bits_to_inorm32_bits(vector[2]), f32_bits_to_inorm32_bits(vector[3])], bytes, *offset as usize),
                    VectorBufferWriteType::V4(BufferWriteType::UNorm8) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u8(&[f32_bits_to_unorm8_bits(vector[0]), f32_bits_to_unorm8_bits(vector[1]), f32_bits_to_unorm8_bits(vector[2]), f32_bits_to_unorm8_bits(vector[3])], bytes, *offset as usize),
                    VectorBufferWriteType::V4(BufferWriteType::UNorm16) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u16(&[f32_bits_to_unorm16_bits(vector[0]), f32_bits_to_unorm16_bits(vector[1]), f32_bits_to_unorm16_bits(vector[2]), f32_bits_to_unorm16_bits(vector[3])], bytes, *offset as usize),
                    VectorBufferWriteType::V4(BufferWriteType::UNorm32) => |bytes: &mut [u8], offset: &u32, vector: [u32; 4]|
                        write_bytes_u32(&[f32_bits_to_unorm32_bits(vector[0]), f32_bits_to_unorm32_bits(vector[1]), f32_bits_to_unorm32_bits(vector[2]), f32_bits_to_unorm32_bits(vector[3])], bytes, *offset as usize),
                };
                todo!()
            },
            ShaderInstruction::ReadBufferToScalar { data_type, scalar, offset, addr_src_u32, buffer } => {
                let buffer = &buffer_modules[*buffer as usize];
                let bytes = buffer.bytes();
                let read_fn = match data_type {
                    BufferReadType::D8  => |bytes: &[u8], offset: usize| read_bytes_u8(bytes, offset) as u32,
                    BufferReadType::D16 => |bytes: &[u8], offset: usize| read_bytes_u16(bytes, offset) as u32,
                    BufferReadType::D32 => |bytes: &[u8], offset: usize| read_bytes_u32(bytes, offset),
                };
                let scalar_register = self.write_scalar_register(*scalar, run_context)?;
                match addr_src_u32 {
                    Some(src_register_addr) => {
                        match self.read_scalar_register(*src_register_addr, run_context) {
                            RegisterRead::Core(addr_register_list) => {
                                (0..CORE_COUNT).for_each(|i| {
                                    scalar_register[i] = read_fn(bytes, *offset as usize + addr_register_list[i] as usize);
                                })
                            },
                            RegisterRead::Uniform(addr_register) => {
                                let value = read_fn(bytes, *offset as usize + *addr_register as usize);
                                scalar_register.fill(value);
                            }
                        }
                    },
                    None => {
                        let value = read_fn(bytes, *offset as usize);
                        scalar_register.fill(value);
                    }
                }
            },
            ShaderInstruction::WriteScalarToBuffer { data_type, scalar, offset, addr_dst_u32, buffer } => {
                let buffer = &mut buffer_modules[*buffer as usize];
                let bytes = buffer.bytes_mut();
                todo!()
            },
            ShaderInstruction::LoadTextureVector { src_xy_u32, dst, texture } => todo!(),
            ShaderInstruction::LoadTextureScalar { src_xy_u32, channel, dst, texture } => todo!(),
            ShaderInstruction::ScalarUnaryOp { src, dst, op } => todo!(),
            ShaderInstruction::ScalarBinaryOp { src_a, src_b, dst, op } => todo!(),
            ShaderInstruction::VectorComponentwiseScalarUnaryOp { src, dst, op } => todo!(),
            ShaderInstruction::VectorComponentwiseScalarBinaryOp { src_a, src_b, dst, op } => todo!(),
            ShaderInstruction::VectorComponentwiseScalarTernaryOp { src_a, src_b, src_c, dst, op } => todo!(),
            ShaderInstruction::VectorComponentwiseScalarUnaryOpMasked { src, dst, op, mask } => todo!(),
            ShaderInstruction::VectorComponentwiseScalarBinaryOpMasked { src_a, src_b, dst, op, mask } => todo!(),
            ShaderInstruction::VectorComponentwiseScalarTernaryOpMasked { src_a, src_b, src_c, dst, op, mask } => todo!(),
            ShaderInstruction::VectorToVectorUnaryOp { src, dst, op } => todo!(),
            ShaderInstruction::VectorToScalarUnaryOp { src, dst, op } => todo!(),
        }
        Some(())
    }

    fn read_scalar_register<'i, 'o>(&self, address: RegisterAddress<Scalar>, run_context: &mut ShadingUnitRunContext<'i, 'o>) -> RegisterRead<'static, u32> {
        {
            let read_ref = match address {
                RegisterAddress::Constant(index, ..) => RegisterRead::Uniform(&run_context.scalar_constant_array[index as usize]),
                RegisterAddress::Input   (index, ..) => RegisterRead::Core(&run_context.scalar_input_array[index as usize]),
                RegisterAddress::Local   (index, ..) => RegisterRead::Core(&self.scalar_locals[index as usize]),
                RegisterAddress::Output  (index, ..) => RegisterRead::Core(&run_context.scalar_output_array[index as usize]),
            };
            read_ref.to_ptr()
        }.to_ref()
    }

    fn read_vector_register<'i, 'o>(&self, address: RegisterAddress<Vector>, run_context: &mut ShadingUnitRunContext<'i, 'o>) -> RegisterRead<'static, [u32; 4]> { 
        {
            let read_ref = match address {
                RegisterAddress::Constant(index, ..) => RegisterRead::Uniform(&run_context.vector_constant_array[index as usize]),
                RegisterAddress::Input   (index, ..) => RegisterRead::Core(&run_context.vector_input_array[index as usize]),
                RegisterAddress::Local   (index, ..) => RegisterRead::Core(&self.vector_locals[index as usize]),
                RegisterAddress::Output  (index, ..) => RegisterRead::Core(&run_context.vector_output_array[index as usize]),
            };
            read_ref.to_ptr()
        }.to_ref()
    }

    fn write_vector_register<'i, 'o>(&mut self, address: RegisterAddress<Vector>, run_context: &mut ShadingUnitRunContext<'i, 'o>) -> Option<&'static mut [[u32; 4]; CORE_COUNT]> {
        let register_ptr = {
            let register_ref = match address {
                RegisterAddress::Local(index, ..) => Some(&mut self.vector_locals[index as usize]),
                RegisterAddress::Output(index, ..) => Some(&mut run_context.vector_output_array[index as usize]),
                _ => None
            };
            register_ref.map(|r| r as *mut _)
        };
        register_ptr.map(|r| unsafe { &mut *r })
    }

    fn write_scalar_register<'i, 'o>(&mut self, address: RegisterAddress<Scalar>, run_context: &mut ShadingUnitRunContext<'i, 'o>) -> Option<&'static mut [u32; CORE_COUNT]> {
        let register_ptr = {
            let register_ref = match address {
                RegisterAddress::Local(index, ..) => Some(&mut self.scalar_locals[index as usize]),
                RegisterAddress::Output(index, ..) => Some(&mut run_context.scalar_output_array[index as usize]),
                _ => None
            };
            register_ref.map(|r| unsafe { (r as *mut _) })
        };
        register_ptr.map(|r| unsafe { &mut *r })
    }
}

fn read_bytes_u8(bytes: &[u8], offset: usize) -> u8 {
    if offset >= bytes.len() {
        0
    } else {
        bytes[offset]
    }
}

fn read_bytes_u16(bytes: &[u8], offset: usize) -> u16 {
    if offset + 2 > bytes.len() {
        0
    } else {
        ((bytes[offset] as u16) << 0) |
        ((bytes[offset] as u16) << 8)
    }
}

fn read_bytes_u32(bytes: &[u8], offset: usize) -> u32 {
    if offset + 4 > bytes.len() {
        0
    } else {
        ((bytes[offset] as u32) <<  0) |
        ((bytes[offset] as u32) <<  8) |
        ((bytes[offset] as u32) << 16) |
        ((bytes[offset] as u32) << 24)
    }
}

fn write_bytes_u8(from: &[u8], to: &mut [u8], offset: usize) {
    if offset + from.len() < to.len() {
        to[offset..(offset + from.len())].copy_from_slice(from);
    }
}

fn write_bytes_u16(from: &[u16], to: &mut [u8], offset: usize) {
    if offset + from.len() * 2 < to.len() {
        for i in 0..from.len() {
            to[offset + i * 2 + 0] = (from[i] >> 0) as u8;
            to[offset + i * 2 + 1] = (from[i] >> 8) as u8;
        }
    }
}

fn write_bytes_u32(from: &[u32], to: &mut [u8], offset: usize) {
    if offset + from.len() * 4 < to.len() {
        for i in 0..from.len() {
            to[offset + i * 2 + 0] = (from[i] >>  0) as u8;
            to[offset + i * 2 + 1] = (from[i] >>  8) as u8;
            to[offset + i * 2 + 1] = (from[i] >> 16) as u8;
            to[offset + i * 2 + 1] = (from[i] >> 24) as u8;
        }
    }
}

fn f32_bits_to_inorm8_bits(bits: u32) -> u8 {
    (f32::from_bits(bits) * std::i8::MAX as f32) as i32 as i8 as u8
}

fn f32_bits_to_unorm8_bits(bits: u32) -> u8 {
    (f32::from_bits(bits) * std::u8::MAX as f32) as u32 as u8
}

fn f32_bits_to_inorm16_bits(bits: u32) -> u16 {
    (f32::from_bits(bits) * std::i16::MAX as f32) as i32 as i16 as u16
}

fn f32_bits_to_unorm16_bits(bits: u32) -> u16 {
    (f32::from_bits(bits) * std::u16::MAX as f32) as u32 as u16
}

fn f32_bits_to_inorm32_bits(bits: u32) -> u32 {
    (f32::from_bits(bits) * std::i32::MAX as f32) as i32 as u32
}

fn f32_bits_to_unorm32_bits(bits: u32) -> u32 {
    (f32::from_bits(bits) * std::u32::MAX as f32) as u32
}

