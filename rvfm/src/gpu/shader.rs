use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Rem;

use super::types::PixelDataLayout;
use super::buffer::BufferModule;
use super::texture::TextureModule;
pub trait RegisterType {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Compute
}

impl ShaderType {
    pub fn from_u8(x: u8) -> Option<Self> {
        match x {
            0 => Some(Self::Vertex),
            1 => Some(Self::Fragment),
            2 => Some(Self::Compute),
            _ => None
        }
    }
}

pub struct ShaderModule {
    pub instruction_buffer: Box<[ShaderInstruction]>,
    pub instruction_count: usize,
    pub shader_type: ShaderType
}

#[derive(Clone, Debug)]
pub struct ResourceMap {
    pub texture: [u8;  64],
    pub buffer:  [u8; 256],
}

impl Default for ResourceMap {
    fn default() -> Self {
        let mut value = Self {
            texture: [0; 64],
            buffer: [0; 256]
        };
        value.texture.iter_mut().enumerate().for_each(|(i, x)| *x = i as u8);
        value.buffer.iter_mut().enumerate().for_each(|(i, x)| *x = i as u8);
        value
    }
}

impl Default for ShaderModule {
    fn default() -> Self {
        Self {
            instruction_buffer: vec![ShaderInstruction::Nop; 1024].into_boxed_slice(),
            instruction_count: 0,
            shader_type: ShaderType::Vertex,
        }
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShaderCardinality {
    Scalar,
    V2,
    V3,
    V4
}

impl ShaderCardinality {
    pub fn from_u8(x: u8) -> Option<ShaderCardinality> {
        Some(match x {
            0 => ShaderCardinality::Scalar,
            1 => ShaderCardinality::V2,
            2 => ShaderCardinality::V3,
            3 => ShaderCardinality::V4,
            _ => None?
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShaderInputType {
    UIntFromU8,
    UIntFromU16,
    UIntFromU32,
    IntFromI8,
    IntFromI16,
    IntFromI32,
    F32FromU8,
    F32FromU16,
    F32FromU32,
    F32FromI8,
    F32FromI16,
    F32FromI32,
    F32FromUNorm8,
    F32FromUNorm16,
    F32FromUNorm32,
    F32FromINorm8,
    F32FromINorm16,
    F32FromINorm32,
    F32FromF32,
}

impl ShaderInputType {
    pub fn from_u8(x: u8) -> Option<Self> {
        Some(match x {
            0x00 => Self::UIntFromU8,
            0x01 => Self::UIntFromU16,
            0x02 => Self::UIntFromU32,
            0x03 => Self::IntFromI8,
            0x04 => Self::IntFromI16,
            0x05 => Self::IntFromI32,
            0x06 => Self::F32FromU8,
            0x07 => Self::F32FromU16,
            0x08 => Self::F32FromU32,
            0x09 => Self::F32FromI8,
            0x0A => Self::F32FromI16,
            0x0B => Self::F32FromI32,
            0x0C => Self::F32FromUNorm8,
            0x0D => Self::F32FromUNorm16,
            0x0E => Self::F32FromUNorm32,
            0x0F => Self::F32FromINorm8,
            0x10 => Self::F32FromINorm16,
            0x11 => Self::F32FromINorm32,
            0x12 => Self::F32FromF32,
            _ => None?
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ShaderConstantAssignment {
    pub constant: u8,
    pub source_buffer: u8,
    pub offset: u32,
    pub t: ShaderInputType,
    pub c: ShaderCardinality,
}

#[derive(Eq, Copy, Clone, Debug)]
pub struct Vector;
impl RegisterType for Vector {}

impl PartialEq for Vector {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Eq, Copy, Clone, Debug)]
pub struct Scalar;
impl RegisterType for Scalar {}

impl PartialEq for Scalar {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RegisterAddress<Type: RegisterType> {
    Local(u8, PhantomData<Type>),
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

#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BufferReadType {
    D8,
    D16,
    D32
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VectorBufferReadType {
    Scalar(BufferReadType),
    V2(BufferReadType),
    V3(BufferReadType),
    V4(BufferReadType)
}

#[allow(unused)]
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

#[allow(unused)]
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

#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TextureLoadType {
    F32FromINorm,
    F32FromUNorm,
    F32FromInt,
    F32FromUInt,
    F32FromF32,
    I32FromInt,
    I32FromUInt,
    I32FromF32,
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OpDataTypeConversion {
    F32ToI32,
    F32ToU32,
    I32toF32,
    U32ToF32,
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScalarUnaryOp {
    Convert     ( OpDataTypeConversion ),
    Negative    ( OpDataType           ),
    Sign        ( OpDataType           ),

    // F32 only
    Reciporocal,
    Sin        ,
    Cos        ,
    Tan        ,
    ASin       ,
    ACos       ,
    Atan       ,
    Ln         ,
    Exp        ,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Comparison {
    Equal,
    NotEqual,
    GreaterThan,
    LessThanOrEqual,
}

impl Comparison {
    pub fn from_u8(x: u8) -> Option<Self> {
        Some(match x {
            0x00 => Self::Equal,
            0x01 => Self::NotEqual,
            0x02 => Self::GreaterThan,
            0x03 => Self::LessThanOrEqual,
            _ => None?
        })
    }
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScalarBinaryOp {
    Compare         ( OpDataType, Comparison ),
    Add             ( OpDataType ),
    Subtract        ( OpDataType ),
    Multiply        ( OpDataType ),
    Divide          ( OpDataType ),
    Modulo          ( OpDataType ),
    // F32 only
    Atan2,
    // I32 as U32 Only
    CompareUnsigned ( Comparison ),
    And,
    AndNot,
    Or,
    Xor
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScalarTernaryOp {
    Fma  ( OpDataType ),
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VectorToVectorUnaryOp {
    Normalize2,
    Normalize3,
    Normalize4,
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VectorToScalarUnaryOp {
    Magnitude2,
    Magnitude3,
    Magnitude4,
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VectorToVectorBinaryOp {
    CrossProduct,
}

#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    ConditionallyCopyVectorRegister {
        cond: RegisterAddress<Scalar>,
        from: RegisterAddress<Vector>,
        to: RegisterAddress<Vector>
    },
    ConditionallyCopyScalarRegister {
        cond: RegisterAddress<Scalar>,
        from: RegisterAddress<Scalar>,
        to: RegisterAddress<Scalar>
    },
    CopyVectorComponentToScalar {
        channel: VectorChannel,
        from: RegisterAddress<Vector>,
        to: RegisterAddress<Scalar>
    },
    ConditionallyCopyVectorComponentToScalar {
        cond: RegisterAddress<Scalar>,
        channel: VectorChannel,
        from: RegisterAddress<Vector>,
        to: RegisterAddress<Scalar>
    },
    CopyScalarToVectorComponent {
        channel: VectorChannel,
        from: RegisterAddress<Scalar>,
        to: RegisterAddress<Vector>
    },
    ConditionallyCopyScalarToVectorComponent {
        cond: RegisterAddress<Scalar>,
        channel: VectorChannel,
        from: RegisterAddress<Scalar>,
        to: RegisterAddress<Vector>
    },
    CopyScalarToVectorComponentsMasked {
        from: RegisterAddress<Scalar>,
        to: RegisterAddress<Vector>,
        mask: u8
    },
    ConditionallyCopyScalarToVectorComponentsMasked {
        cond: RegisterAddress<Scalar>,
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
    ConditionallyWriteVectorToBuffer {
        cond: RegisterAddress<Scalar>,
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
    ConditionallyWriteScalarToBuffer {
        cond: RegisterAddress<Scalar>,
        data_type: BufferWriteType,
        scalar: RegisterAddress<Scalar>,
        offset: u32,
        addr_dst_u32: Option<RegisterAddress<Scalar>>,
        buffer: u8,
    },
    LoadTextureVector {
        src_xy_u32: RegisterAddress<Vector>,
        dst: RegisterAddress<Vector>,
        load_type: TextureLoadType,
        texture: u8,
    },
    LoadTextureScalar {
        src_xy_u32: RegisterAddress<Vector>,
        channel: VectorChannel,
        dst: RegisterAddress<Scalar>,
        load_type: TextureLoadType,
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
    MatrixMultiply4x4V4 {
        a0: RegisterAddress<Vector>,
        a1: RegisterAddress<Vector>,
        a2: RegisterAddress<Vector>,
        a3: RegisterAddress<Vector>,
        x: RegisterAddress<Vector>,
        dest: RegisterAddress<Vector>,
    },
    Nop,
}

pub const CORE_COUNT: usize = 0x1000;
pub const STACK_SIZE: usize = 0x400;
pub const LOCAL_COUNT: usize = 0x20;
pub const INPUT_OUTPUT_COUNT: usize = 0x100;
pub const CONST_COUNT: usize = 0x100;

type InputArrayRef<'a, T> = &'a mut [[T; CORE_COUNT]; INPUT_OUTPUT_COUNT];
type ConstArrayRef<'a, T> = &'a mut [T; CONST_COUNT];
type OutputArrayRef<'a, T> = &'a mut [[T; CORE_COUNT]; INPUT_OUTPUT_COUNT];

type InputOutputArray<T> = [[T; CORE_COUNT]; INPUT_OUTPUT_COUNT];
type ConstantArray<T> = [T; CONST_COUNT];


pub struct ShadingUnitConstantArray {
    pub scalar_constant_array: ConstantArray<u32>,
    pub vector_constant_array: ConstantArray<[u32; 4]>,
}

impl ShadingUnitConstantArray {
    pub fn new() -> Self {
        Self {
            scalar_constant_array: [ 0u32    ; CONST_COUNT],
            vector_constant_array: [[0u32; 4]; CONST_COUNT],
        }
    }
}

pub struct ShadingUnitIOArray {
    pub scalar_array: InputOutputArray<u32>,
    pub vector_array: InputOutputArray<[u32; 4]>,
}

impl ShadingUnitIOArray {
    pub fn new() -> Self {
        Self {
            scalar_array: [[ 0u32;     CORE_COUNT]; INPUT_OUTPUT_COUNT],
            vector_array: [[[0u32; 4]; CORE_COUNT]; INPUT_OUTPUT_COUNT],
        }
    }
}

pub struct ShadingUnitIOArrays(pub [ShadingUnitIOArray; 3]);

impl ShadingUnitIOArrays {
    pub fn new() -> Box<Self> {
        let box_uninit = Box::new_zeroed();
        unsafe { Box::<MaybeUninit<Self>>::assume_init(box_uninit) }
    }
}

pub fn setup_shader_constants<'a>(constant_array: &'a mut ShadingUnitConstantArray, constants: &[ShaderConstantAssignment], resource_map: &ResourceMap, buffer_modules: &mut [BufferModule; 256]) {
    for constant_assignment in constants.iter() {
        let buffer_index = resource_map.buffer[constant_assignment.source_buffer as usize] as usize;
        let bytes = buffer_modules[buffer_index].bytes();
        let read_fn = match constant_assignment.t {
            ShaderInputType::UIntFromU8     => |bytes: &[u8], offset: usize|  read_bytes_u8 (bytes, offset) as u32                                         ,
            ShaderInputType::UIntFromU16    => |bytes: &[u8], offset: usize|  read_bytes_u16(bytes, offset) as u32                                         ,
            ShaderInputType::F32FromF32  |
            ShaderInputType::UIntFromU32 |
            ShaderInputType::IntFromI32     => |bytes: &[u8], offset: usize|  read_bytes_u32(bytes, offset),
            ShaderInputType::IntFromI8      => |bytes: &[u8], offset: usize|  read_bytes_u8 (bytes, offset) as i8  as i32 as u32                           ,
            ShaderInputType::IntFromI16     => |bytes: &[u8], offset: usize|  read_bytes_u16(bytes, offset) as i16 as i32 as u32                           ,
            ShaderInputType::F32FromI8      => |bytes: &[u8], offset: usize| (read_bytes_u8 (bytes, offset) as i8  as f32                       ).to_bits(),
            ShaderInputType::F32FromI16     => |bytes: &[u8], offset: usize| (read_bytes_u16(bytes, offset) as i16 as f32                       ).to_bits(),
            ShaderInputType::F32FromI32     => |bytes: &[u8], offset: usize| (read_bytes_u32(bytes, offset) as i32 as f32                       ).to_bits(),
            ShaderInputType::F32FromU8      => |bytes: &[u8], offset: usize| (read_bytes_u8 (bytes, offset)        as f32                       ).to_bits(),
            ShaderInputType::F32FromU16     => |bytes: &[u8], offset: usize| (read_bytes_u16(bytes, offset)        as f32                       ).to_bits(),
            ShaderInputType::F32FromU32     => |bytes: &[u8], offset: usize| (read_bytes_u32(bytes, offset)        as f32                       ).to_bits(),
            ShaderInputType::F32FromUNorm8  => |bytes: &[u8], offset: usize| (read_bytes_u8 (bytes, offset)        as f32 / std::u8::MAX  as f32).to_bits(),
            ShaderInputType::F32FromUNorm16 => |bytes: &[u8], offset: usize| (read_bytes_u16(bytes, offset)        as f32 / std::u16::MAX as f32).to_bits(),
            ShaderInputType::F32FromUNorm32 => |bytes: &[u8], offset: usize| (read_bytes_u32(bytes, offset)        as f32                       ).to_bits(),
            ShaderInputType::F32FromINorm8  => |bytes: &[u8], offset: usize| (read_bytes_u8 (bytes, offset) as i8  as f32 / std::i8::MAX  as f32).to_bits(),
            ShaderInputType::F32FromINorm16 => |bytes: &[u8], offset: usize| (read_bytes_u16(bytes, offset) as i16 as f32 / std::i16::MAX as f32).to_bits(),
            ShaderInputType::F32FromINorm32 => |bytes: &[u8], offset: usize| (read_bytes_u32(bytes, offset) as i32 as f32 / std::i32::MAX as f32).to_bits(),
        };
        let element_size = match constant_assignment.t {
            ShaderInputType::F32FromF32 |
            ShaderInputType::F32FromI32 |
            ShaderInputType::F32FromU32 |
            ShaderInputType::F32FromINorm32 |
            ShaderInputType::F32FromUNorm32 |
            ShaderInputType::IntFromI32 |
            ShaderInputType::UIntFromU32 => 4,
            ShaderInputType::F32FromI16 |
            ShaderInputType::F32FromU16 |
            ShaderInputType::F32FromINorm16 |
            ShaderInputType::F32FromUNorm16 |
            ShaderInputType::IntFromI16 |
            ShaderInputType::UIntFromU16 => 2,
            ShaderInputType::F32FromI8 |
            ShaderInputType::F32FromU8 |
            ShaderInputType::F32FromUNorm8 |
            ShaderInputType::F32FromINorm8 |
            ShaderInputType::IntFromI8 |
            ShaderInputType::UIntFromU8 => 1
        };
        match constant_assignment.c {
            ShaderCardinality::Scalar => {
                constant_array.scalar_constant_array[constant_assignment.constant as usize] = read_fn(bytes, constant_assignment.offset as usize);
            },
            ShaderCardinality::V2 => {
                constant_array.vector_constant_array[constant_assignment.constant as usize][0] = read_fn(bytes, constant_assignment.offset as usize + element_size * 0);
                constant_array.vector_constant_array[constant_assignment.constant as usize][1] = read_fn(bytes, constant_assignment.offset as usize + element_size * 1);
            },
            ShaderCardinality::V3 => {
                constant_array.vector_constant_array[constant_assignment.constant as usize][0] = read_fn(bytes, constant_assignment.offset as usize + element_size * 0);
                constant_array.vector_constant_array[constant_assignment.constant as usize][1] = read_fn(bytes, constant_assignment.offset as usize + element_size * 1);
                constant_array.vector_constant_array[constant_assignment.constant as usize][2] = read_fn(bytes, constant_assignment.offset as usize + element_size * 2);
            },
            ShaderCardinality::V4 => {
                constant_array.vector_constant_array[constant_assignment.constant as usize][0] = read_fn(bytes, constant_assignment.offset as usize + element_size * 0);
                constant_array.vector_constant_array[constant_assignment.constant as usize][1] = read_fn(bytes, constant_assignment.offset as usize + element_size * 1);
                constant_array.vector_constant_array[constant_assignment.constant as usize][2] = read_fn(bytes, constant_assignment.offset as usize + element_size * 2);
                constant_array.vector_constant_array[constant_assignment.constant as usize][3] = read_fn(bytes, constant_assignment.offset as usize + element_size * 3);
            },
        };
    }
}

#[derive(Debug)]
pub struct ShadingUnitRunContext<'a> {
    pub scalar_input_array: InputArrayRef<'a, u32>,
    pub vector_input_array: InputArrayRef<'a, [u32; 4]>,
    pub scalar_constant_array: ConstArrayRef<'a, u32>,
    pub vector_constant_array: ConstArrayRef<'a, [u32; 4]>,
    pub scalar_output_array: OutputArrayRef<'a, u32>,
    pub vector_output_array: OutputArrayRef<'a, [u32; 4]>,
}

impl ShadingUnitRunContext<'_> {
    pub fn new<'a>(constant_array: &'a mut ShadingUnitConstantArray, input_array: &'a mut ShadingUnitIOArray, output_array: &'a mut ShadingUnitIOArray) -> ShadingUnitRunContext<'a>{
        ShadingUnitRunContext {
            scalar_input_array: &mut input_array.scalar_array,
            vector_input_array: &mut input_array.vector_array,
            scalar_constant_array: &mut constant_array.scalar_constant_array,
            vector_constant_array: &mut constant_array.vector_constant_array,
            scalar_output_array: &mut output_array.scalar_array,
            vector_output_array: &mut output_array.vector_array,
        }
    }
}

#[derive(Debug)]
pub struct ShadingUnitContext {
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
    pub fn new() -> Box<Self> {
        let box_uninit = Box::new_zeroed();
        unsafe { Box::<MaybeUninit<Self>>::assume_init(box_uninit) }
    }

    pub fn run_instruction<'io, 'rc>(&mut self, n: usize, instruction: &ShaderInstruction, run_context: &'rc mut ShadingUnitRunContext<'io>, buffer_modules: &mut [BufferModule; 256], texture_modules: &mut [TextureModule; 64], resource_map: &ResourceMap) -> Option<()> {
        match instruction {
            ShaderInstruction::Nop => {},
            ShaderInstruction::PushVector(register) => {
                let source_register = self.read_vector_register(*register, run_context);
                let stack_slot = &mut self.vector_stack_array[self.vector_stack_size];
                match source_register {
                    RegisterRead::Core(register_list) => stack_slot[0..n].copy_from_slice(&register_list[0..n]),
                    RegisterRead::Uniform(register) => stack_slot[0..n].fill(*register),
                }
                self.vector_stack_size += 1;
            },
            ShaderInstruction::PushScalar(register) => {
                let source_register = self.read_scalar_register(*register, run_context);
                let stack_slot = &mut self.scalar_stack_array[self.vector_stack_size];
                match source_register {
                    RegisterRead::Core(register_list) => stack_slot[0..n].copy_from_slice(&register_list[0..n]),
                    RegisterRead::Uniform(register) => stack_slot[0..n].fill(*register),
                }
                self.scalar_stack_size += 1;
            },
            ShaderInstruction::PopVector(register) => {
                let register_list = self.write_vector_register(*register, run_context)?;
                self.vector_stack_size -= 1;
                let stack_slot = &self.vector_stack_array[self.vector_stack_size];
                register_list[0..n].copy_from_slice(&stack_slot[0..n]);
            },
            ShaderInstruction::PopScalar(register) => {
                let register_list = self.write_scalar_register(*register, run_context)?;
                self.scalar_stack_size -= 1;
                let stack_slot = &self.scalar_stack_array[self.scalar_stack_size];
                register_list[0..n].copy_from_slice(&stack_slot[0..n]);
            },
            ShaderInstruction::CopyVectorRegister { from, to } => {
                if *from == *to { None? }
                let from_register = self.read_vector_register(*from, run_context);
                let to_register_list = self.write_vector_register(*to, run_context)?;
                match from_register {
                    RegisterRead::Core(from_register_list) => to_register_list[0..n].copy_from_slice(&from_register_list[0..n]),
                    RegisterRead::Uniform(from_register) => to_register_list[0..n].fill(*from_register),
                }
            },
            ShaderInstruction::ConditionallyCopyVectorRegister { cond, from, to } => {
                if *from == *to { None? }
                let cond_register = self.read_scalar_register(*cond, run_context);
                let from_register = self.read_vector_register(*from, run_context);
                let to_register_list = self.write_vector_register(*to, run_context)?;
                match cond_register {
                    RegisterRead::Uniform(cond_register) => if *cond_register != 0 {
                        match from_register {
                            RegisterRead::Core(from_register_list) => to_register_list[0..n].copy_from_slice(&from_register_list[0..n]),
                            RegisterRead::Uniform(from_register) => to_register_list[0..n].fill(*from_register),
                        }
                    },
                    RegisterRead::Core(cond_register_list) => {
                        match from_register {
                            RegisterRead::Core(from_register_list) => (0..n).for_each(|i| if cond_register_list[i] != 0 { to_register_list[i] = from_register_list[i]; }),
                            RegisterRead::Uniform(from_register) => (0..n).for_each(|i| if cond_register_list[i] != 0 { to_register_list[i] = *from_register; }),
                        }
                    }
                }
            },
            ShaderInstruction::CopyScalarRegister { from, to } => {
                if *from == *to { None? }
                let from_register = self.read_scalar_register(*from, run_context);
                let to_register_list = self.write_scalar_register(*to, run_context)?;
                match from_register {
                    RegisterRead::Core(from_register_list) => to_register_list[0..n].copy_from_slice(&from_register_list[0..n]),
                    RegisterRead::Uniform(from_register) => to_register_list[0..n].fill(*from_register),
                }
            },
            ShaderInstruction::ConditionallyCopyScalarRegister { cond, from, to } => {
                if *from == *to { None? }
                let cond_register = self.read_scalar_register(*cond, run_context);
                let from_register = self.read_scalar_register(*from, run_context);
                let to_register_list = self.write_scalar_register(*to, run_context)?;
                match from_register {
                    RegisterRead::Core(from_register_list) => to_register_list[0..n].copy_from_slice(&from_register_list[0..n]),
                    RegisterRead::Uniform(from_register) => to_register_list[0..n].fill(*from_register),
                }
                match cond_register {
                    RegisterRead::Uniform(cond_register) => if *cond_register != 0 {
                        match from_register {
                            RegisterRead::Core(from_register_list) => to_register_list[0..n].copy_from_slice(&from_register_list[0..n]),
                            RegisterRead::Uniform(from_register) => to_register_list[0..n].fill(*from_register),
                        }
                    },
                    RegisterRead::Core(cond_register_list) => {
                        match from_register {
                            RegisterRead::Core(from_register_list) => (0..n).for_each(|i| if cond_register_list[i] != 0 { to_register_list[i] = from_register_list[i]; }),
                            RegisterRead::Uniform(from_register) => (0..n).for_each(|i| if cond_register_list[i] != 0 { to_register_list[i] = *from_register; }),
                        }
                    }
                }
            },
            ShaderInstruction::CopyVectorComponentToScalar { channel, from, to } => {
                let from_register = self.read_vector_register(*from, run_context);
                let to_register_list = self.write_scalar_register(*to, run_context)?;
                match from_register {
                    RegisterRead::Core(from_register_list) =>
                        (0..n)
                            .for_each(|i| to_register_list[i] = from_register_list[i][*channel as usize]),
                    RegisterRead::Uniform(from_register) =>
                        (0..n)
                            .for_each(|i| to_register_list[i] = from_register[*channel as usize]),
                }
            },
            ShaderInstruction::ConditionallyCopyVectorComponentToScalar { cond, channel, from, to } => {
                let cond_register = self.read_scalar_register(*cond, run_context);
                let from_register = self.read_vector_register(*from, run_context);
                let to_register_list = self.write_scalar_register(*to, run_context)?;
                match cond_register {
                    RegisterRead::Uniform(cond_register) => if *cond_register != 0 {
                        match from_register {
                            RegisterRead::Core(from_register_list) =>
                                (0..n)
                                    .for_each(|i| to_register_list[i] = from_register_list[i][*channel as usize]),
                            RegisterRead::Uniform(from_register) =>
                                (0..n)
                                    .for_each(|i| to_register_list[i] = from_register[*channel as usize]),
                        }
                    },
                    RegisterRead::Core(cond_register_list) => {
                        match from_register {
                            RegisterRead::Core(from_register_list) =>
                                (0..n)
                                    .for_each(|i| if cond_register_list[i] != 0 { to_register_list[i] = from_register_list[i][*channel as usize] }),
                            RegisterRead::Uniform(from_register) =>
                                (0..n)
                                    .for_each(|i| if cond_register_list[i] != 0 { to_register_list[i] = from_register[*channel as usize] }),
                        }
                    }
                }
                
            },
            ShaderInstruction::CopyScalarToVectorComponent { channel, from, to } => {
                let from_register = self.read_scalar_register(*from, run_context);
                let to_register_list = self.write_vector_register(*to, run_context)?;
                match from_register {
                    RegisterRead::Core(from_register_list) =>
                        (0..n)
                            .for_each(|i| to_register_list[i][*channel as usize] = from_register_list[i]),
                    RegisterRead::Uniform(from_register) =>
                        (0..n)
                            .for_each(|i| to_register_list[i][*channel as usize] = *from_register),
                }
            },
            ShaderInstruction::ConditionallyCopyScalarToVectorComponent { cond, channel, from, to } => {
                let cond_register = self.read_scalar_register(*cond, run_context);
                let from_register = self.read_scalar_register(*from, run_context);
                let to_register_list = self.write_vector_register(*to, run_context)?;
                match cond_register {
                    RegisterRead::Uniform(cond_register) => if *cond_register != 0 {
                        match from_register {
                            RegisterRead::Core(from_register_list) =>
                                (0..n)
                                    .for_each(|i| to_register_list[i][*channel as usize] = from_register_list[i]),
                            RegisterRead::Uniform(from_register) =>
                                (0..n)
                                    .for_each(|i| to_register_list[i][*channel as usize] = *from_register),
                        }
                    },
                    RegisterRead::Core(cond_register_list) => {
                        match from_register {
                            RegisterRead::Core(from_register_list) =>
                                (0..n)
                                    .for_each(|i| if cond_register_list[i] != 0 { to_register_list[i][*channel as usize] = from_register_list[i]; }),
                            RegisterRead::Uniform(from_register) =>
                                (0..n)
                                    .for_each(|i| if cond_register_list[i] != 0 { to_register_list[i][*channel as usize] = *from_register }),
                        }
                    },
                }
            },
            ShaderInstruction::CopyScalarToVectorComponentsMasked { from, to, mask } => {
                let from_register = self.read_scalar_register(*from, run_context);
                let to_register_list = self.write_vector_register(*to, run_context)?;
                match from_register {
                    RegisterRead::Core(from_register_list) =>
                        (0..4).for_each(|c| {
                            if (mask & (1 << c)) != 0 {
                                (0..n).for_each(|i| {
                                    to_register_list[i][c] = from_register_list[i];
                                })
                            }
                        }),
                    RegisterRead::Uniform(from_register) =>
                        (0..4).for_each(|c| {
                            if (mask & (1 << c)) != 0 {
                                (0..n).for_each(|i| {
                                    to_register_list[i][c] = *from_register;
                                })
                            }
                        }),
                }
            },
            ShaderInstruction::ReadBufferToVector { data_type, vector, offset, addr_src_u32, buffer } => {
                let buffer_number = resource_map.buffer[*buffer as usize] as usize;
                let buffer = &buffer_modules[buffer_number];
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
                                (0..n).for_each(|i| {
                                    vector_register[i] = read_fn(buffer_bytes, *offset as usize + addr_register_list[i] as usize);
                                })
                            },
                            RegisterRead::Uniform(addr_register) => {
                                let value = read_fn(buffer_bytes, *offset as usize + *addr_register as usize);
                                vector_register[0..n].fill(value);
                            }
                        }
                    },
                    None => {
                        let value = read_fn(buffer_bytes, *offset as usize);
                        vector_register[0..n].fill(value);
                    }
                }
            },
            ShaderInstruction::WriteVectorToBuffer{ data_type, src, offset, addr_src_u32, buffer } => {
                let buffer_number = resource_map.buffer[*buffer as usize] as usize;
                let buffer = &mut buffer_modules[buffer_number];
                let bytes = buffer.bytes_mut();
                let from_register = self.read_vector_register(*src, run_context);
                let write_fn = match *data_type {
                    VectorBufferWriteType::Scalar(BufferWriteType::I8 ) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u8 (&[vector[0] as  i8 as  u8], bytes, offset),
                    VectorBufferWriteType::Scalar(BufferWriteType::I16) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u16(&[vector[0] as i16 as u16], bytes, offset),
                    VectorBufferWriteType::Scalar(BufferWriteType::I32) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u32(&[vector[0] as i32 as u32], bytes, offset),
                    VectorBufferWriteType::Scalar(BufferWriteType::U8 ) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u8 (&[vector[0] as  u8], bytes, offset),
                    VectorBufferWriteType::Scalar(BufferWriteType::U16) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u16(&[vector[0] as u16], bytes, offset),
                    VectorBufferWriteType::Scalar(BufferWriteType::U32) |
                    VectorBufferWriteType::Scalar(BufferWriteType::F32) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u32(&[vector[0]], bytes, offset),
                    VectorBufferWriteType::Scalar(BufferWriteType::INorm8) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u8(&[f32_bits_to_inorm8_bits(vector[0])], bytes, offset),
                    VectorBufferWriteType::Scalar(BufferWriteType::INorm16) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u16(&[f32_bits_to_inorm16_bits(vector[0])], bytes, offset),
                    VectorBufferWriteType::Scalar(BufferWriteType::INorm32) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u32(&[f32_bits_to_inorm32_bits(vector[0])], bytes, offset),
                    VectorBufferWriteType::Scalar(BufferWriteType::UNorm8) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u8(&[f32_bits_to_unorm8_bits(vector[0])], bytes, offset),
                    VectorBufferWriteType::Scalar(BufferWriteType::UNorm16) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u16(&[f32_bits_to_unorm16_bits(vector[0])], bytes, offset),
                    VectorBufferWriteType::Scalar(BufferWriteType::UNorm32) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u32(&[f32_bits_to_unorm32_bits(vector[0])], bytes, offset),
                    
                    VectorBufferWriteType::V2(BufferWriteType::I8 ) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u8 (&[vector[0] as  i8 as  u8, vector[1] as  i8 as  u8], bytes, offset),
                    VectorBufferWriteType::V2(BufferWriteType::I16) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u16(&[vector[0] as i16 as u16, vector[1] as i16 as u16], bytes, offset),
                    VectorBufferWriteType::V2(BufferWriteType::I32) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u32(&[vector[0] as i32 as u32, vector[1] as i32 as u32], bytes, offset),
                    VectorBufferWriteType::V2(BufferWriteType::U8 ) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u8 (&[vector[0] as  u8, vector[1] as  u8], bytes, offset),
                    VectorBufferWriteType::V2(BufferWriteType::U16) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u16(&[vector[0] as u16, vector[1] as u16], bytes, offset),
                    VectorBufferWriteType::V2(BufferWriteType::U32) |
                    VectorBufferWriteType::V2(BufferWriteType::F32) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u32(&[vector[0], vector[1]], bytes, offset),
                    VectorBufferWriteType::V2(BufferWriteType::INorm8) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u8(&[f32_bits_to_inorm8_bits(vector[0]), f32_bits_to_inorm8_bits(vector[1])], bytes, offset),
                    VectorBufferWriteType::V2(BufferWriteType::INorm16) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u16(&[f32_bits_to_inorm16_bits(vector[0]), f32_bits_to_inorm16_bits(vector[1])], bytes, offset),
                    VectorBufferWriteType::V2(BufferWriteType::INorm32) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u32(&[f32_bits_to_inorm32_bits(vector[0]), f32_bits_to_inorm32_bits(vector[1])], bytes, offset),
                    VectorBufferWriteType::V2(BufferWriteType::UNorm8) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u8(&[f32_bits_to_unorm8_bits(vector[0]), f32_bits_to_unorm8_bits(vector[1])], bytes, offset),
                    VectorBufferWriteType::V2(BufferWriteType::UNorm16) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u16(&[f32_bits_to_unorm16_bits(vector[0]), f32_bits_to_unorm16_bits(vector[1])], bytes, offset),
                    VectorBufferWriteType::V2(BufferWriteType::UNorm32) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u32(&[f32_bits_to_unorm32_bits(vector[0]), f32_bits_to_unorm32_bits(vector[1])], bytes, offset),

                    VectorBufferWriteType::V3(BufferWriteType::I8 ) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u8 (&[vector[0] as  i8 as  u8, vector[1] as  i8 as  u8, vector[2] as  i8 as  u8], bytes, offset),
                    VectorBufferWriteType::V3(BufferWriteType::I16) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u16(&[vector[0] as i16 as u16, vector[1] as i16 as u16, vector[2] as i16 as u16], bytes, offset),
                    VectorBufferWriteType::V3(BufferWriteType::I32) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u32(&[vector[0] as i32 as u32, vector[1] as i32 as u32, vector[2] as i32 as u32], bytes, offset),
                    VectorBufferWriteType::V3(BufferWriteType::U8 ) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u8 (&[vector[0] as  u8, vector[1] as  u8, vector[2] as  u8], bytes, offset),
                    VectorBufferWriteType::V3(BufferWriteType::U16) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u16(&[vector[0] as u16, vector[1] as u16, vector[2] as u16], bytes, offset),
                    VectorBufferWriteType::V3(BufferWriteType::U32) |
                    VectorBufferWriteType::V3(BufferWriteType::F32) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u32(&[vector[0], vector[1], vector[2]], bytes, offset),
                    VectorBufferWriteType::V3(BufferWriteType::INorm8) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u8(&[f32_bits_to_inorm8_bits(vector[0]), f32_bits_to_inorm8_bits(vector[1]), f32_bits_to_inorm8_bits(vector[2])], bytes, offset),
                    VectorBufferWriteType::V3(BufferWriteType::INorm16) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u16(&[f32_bits_to_inorm16_bits(vector[0]), f32_bits_to_inorm16_bits(vector[1]), f32_bits_to_inorm16_bits(vector[2])], bytes, offset),
                    VectorBufferWriteType::V3(BufferWriteType::INorm32) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u32(&[f32_bits_to_inorm32_bits(vector[0]), f32_bits_to_inorm32_bits(vector[1]), f32_bits_to_inorm32_bits(vector[2])], bytes, offset),
                    VectorBufferWriteType::V3(BufferWriteType::UNorm8) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u8(&[f32_bits_to_unorm8_bits(vector[0]), f32_bits_to_unorm8_bits(vector[1]), f32_bits_to_unorm8_bits(vector[2])], bytes, offset),
                    VectorBufferWriteType::V3(BufferWriteType::UNorm16) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u16(&[f32_bits_to_unorm16_bits(vector[0]), f32_bits_to_unorm16_bits(vector[1]), f32_bits_to_unorm16_bits(vector[2])], bytes, offset),
                    VectorBufferWriteType::V3(BufferWriteType::UNorm32) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u32(&[f32_bits_to_unorm32_bits(vector[0]), f32_bits_to_unorm32_bits(vector[1]), f32_bits_to_unorm32_bits(vector[2])], bytes, offset),

                    VectorBufferWriteType::V4(BufferWriteType::I8 ) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u8 (&[vector[0] as  i8 as  u8, vector[1] as  i8 as  u8, vector[2] as  i8 as  u8, vector[3] as  i8 as  u8], bytes, offset),
                    VectorBufferWriteType::V4(BufferWriteType::I16) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u16(&[vector[0] as i16 as u16, vector[1] as i16 as u16, vector[2] as i16 as u16, vector[3] as i16 as u16], bytes, offset),
                    VectorBufferWriteType::V4(BufferWriteType::I32) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u32(&[vector[0] as i32 as u32, vector[1] as i32 as u32, vector[2] as i32 as u32, vector[3] as i32 as u32], bytes, offset),
                    VectorBufferWriteType::V4(BufferWriteType::U8 ) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u8 (&[vector[0] as  u8, vector[1] as  u8, vector[2] as  u8, vector[3] as  u8], bytes, offset),
                    VectorBufferWriteType::V4(BufferWriteType::U16) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u16(&[vector[0] as u16, vector[1] as u16, vector[2] as u16, vector[3] as u16], bytes, offset),
                    VectorBufferWriteType::V4(BufferWriteType::U32) |
                    VectorBufferWriteType::V4(BufferWriteType::F32) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u32(&[vector[0], vector[1], vector[2], vector[3]], bytes, offset),
                    VectorBufferWriteType::V4(BufferWriteType::INorm8) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u8(&[f32_bits_to_inorm8_bits(vector[0]), f32_bits_to_inorm8_bits(vector[1]), f32_bits_to_inorm8_bits(vector[2]), f32_bits_to_inorm8_bits(vector[3])], bytes, offset),
                    VectorBufferWriteType::V4(BufferWriteType::INorm16) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u16(&[f32_bits_to_inorm16_bits(vector[0]), f32_bits_to_inorm16_bits(vector[1]), f32_bits_to_inorm16_bits(vector[2]), f32_bits_to_inorm16_bits(vector[3])], bytes, offset),
                    VectorBufferWriteType::V4(BufferWriteType::INorm32) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u32(&[f32_bits_to_inorm32_bits(vector[0]), f32_bits_to_inorm32_bits(vector[1]), f32_bits_to_inorm32_bits(vector[2]), f32_bits_to_inorm32_bits(vector[3])], bytes, offset),
                    VectorBufferWriteType::V4(BufferWriteType::UNorm8) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u8(&[f32_bits_to_unorm8_bits(vector[0]), f32_bits_to_unorm8_bits(vector[1]), f32_bits_to_unorm8_bits(vector[2]), f32_bits_to_unorm8_bits(vector[3])], bytes, offset),
                    VectorBufferWriteType::V4(BufferWriteType::UNorm16) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u16(&[f32_bits_to_unorm16_bits(vector[0]), f32_bits_to_unorm16_bits(vector[1]), f32_bits_to_unorm16_bits(vector[2]), f32_bits_to_unorm16_bits(vector[3])], bytes, offset),
                    VectorBufferWriteType::V4(BufferWriteType::UNorm32) => |bytes: &mut [u8], offset: usize, vector: [u32; 4]|
                        write_bytes_u32(&[f32_bits_to_unorm32_bits(vector[0]), f32_bits_to_unorm32_bits(vector[1]), f32_bits_to_unorm32_bits(vector[2]), f32_bits_to_unorm32_bits(vector[3])], bytes, offset),
                };
                match addr_src_u32 {
                    Some(src_register_addr) => {
                        match self.read_scalar_register(*src_register_addr, run_context) {
                            RegisterRead::Core(addr_register_list) => {
                                match from_register {
                                    RegisterRead::Core(from_register_list) => (0..n).for_each(|i| write_fn(bytes, *offset as usize + addr_register_list[i] as usize, from_register_list[i])),
                                    RegisterRead::Uniform(from_register) => (0..n).for_each(|i| write_fn(bytes, *offset as usize + addr_register_list[i] as usize, *from_register)),
                                }
                            },
                            RegisterRead::Uniform(addr_register) => {
                                match from_register {
                                    RegisterRead::Core(from_register_list) => (0..n).for_each(|i| write_fn(bytes, *offset as usize + *addr_register as usize, from_register_list[i])),
                                    RegisterRead::Uniform(from_register) => (0..n).for_each(|_i| write_fn(bytes, *offset as usize + *addr_register as usize, *from_register)),
                                }
                            }
                        }
                    },
                    None => {
                        match from_register {
                            RegisterRead::Core(from_register_list) => (0..n).for_each(|i| write_fn(bytes, *offset as usize, from_register_list[i])),
                            RegisterRead::Uniform(from_register) => (0..n).for_each(|_i| write_fn(bytes, *offset as usize, *from_register)),
                        }
                    }
                }
            },
            ShaderInstruction::ReadBufferToScalar { data_type, scalar, offset, addr_src_u32, buffer } => {
                let buffer_number = resource_map.buffer[*buffer as usize] as usize;
                let buffer = &buffer_modules[buffer_number];
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
                                (0..n).for_each(|i| {
                                    scalar_register[i] = read_fn(bytes, *offset as usize + addr_register_list[i] as usize);
                                })
                            },
                            RegisterRead::Uniform(addr_register) => {
                                let value = read_fn(bytes, *offset as usize + *addr_register as usize);
                                scalar_register[0..n].fill(value);
                            }
                        }
                    },
                    None => {
                        let value = read_fn(bytes, *offset as usize);
                        scalar_register[0..n].fill(value);
                    }
                }
            },
            ShaderInstruction::WriteScalarToBuffer { data_type, scalar, offset, addr_dst_u32, buffer } => {
                let buffer_number = resource_map.buffer[*buffer as usize] as usize;
                let buffer = &mut buffer_modules[buffer_number];
                let bytes = buffer.bytes_mut();
                let from_register = self.read_scalar_register(*scalar, run_context);
                let write_fn = match *data_type {
                    BufferWriteType::I8      => |bytes: &mut [u8], offset: usize, scalar: u32| write_bytes_u8 (&[scalar as i32 as  i8 as  u8], bytes, offset),
                    BufferWriteType::I16     => |bytes: &mut [u8], offset: usize, scalar: u32| write_bytes_u16(&[scalar as i32 as i16 as u16], bytes, offset),
                    BufferWriteType::I32     => |bytes: &mut [u8], offset: usize, scalar: u32| write_bytes_u32(&[scalar as i32 as i32 as u32], bytes, offset),
                    BufferWriteType::U8      => |bytes: &mut [u8], offset: usize, scalar: u32| write_bytes_u8 (&[scalar as  u8], bytes, offset),
                    BufferWriteType::U16     => |bytes: &mut [u8], offset: usize, scalar: u32| write_bytes_u16(&[scalar as u16], bytes, offset),
                    BufferWriteType::U32 |
                    BufferWriteType::F32     => |bytes: &mut [u8], offset: usize, scalar: u32| write_bytes_u32(&[scalar], bytes, offset),
                    BufferWriteType::INorm8  => |bytes: &mut [u8], offset: usize, scalar: u32| write_bytes_u8 (&[f32_bits_to_inorm8_bits (scalar)], bytes, offset),
                    BufferWriteType::INorm16 => |bytes: &mut [u8], offset: usize, scalar: u32| write_bytes_u16(&[f32_bits_to_inorm16_bits(scalar)], bytes, offset),
                    BufferWriteType::INorm32 => |bytes: &mut [u8], offset: usize, scalar: u32| write_bytes_u32(&[f32_bits_to_inorm32_bits(scalar)], bytes, offset),
                    BufferWriteType::UNorm8  => |bytes: &mut [u8], offset: usize, scalar: u32| write_bytes_u8 (&[f32_bits_to_unorm8_bits (scalar)], bytes, offset),
                    BufferWriteType::UNorm16 => |bytes: &mut [u8], offset: usize, scalar: u32| write_bytes_u16(&[f32_bits_to_unorm16_bits(scalar)], bytes, offset),
                    BufferWriteType::UNorm32 => |bytes: &mut [u8], offset: usize, scalar: u32| write_bytes_u32(&[f32_bits_to_unorm32_bits(scalar)], bytes, offset),   
                };
                match addr_dst_u32 {
                    Some(src_register_addr) => {
                        match self.read_scalar_register(*src_register_addr, run_context) {
                            RegisterRead::Core(addr_register_list) => {
                                match from_register {
                                    RegisterRead::Core(from_register_list) => (0..n).for_each(|i| write_fn(bytes, *offset as usize + addr_register_list[i] as usize, from_register_list[i])),
                                    RegisterRead::Uniform(from_register) => (0..n).for_each(|i| write_fn(bytes, *offset as usize + addr_register_list[i] as usize, *from_register)),
                                }
                            },
                            RegisterRead::Uniform(addr_register) => {
                                match from_register {
                                    RegisterRead::Core(from_register_list) => (0..n).for_each(|i| write_fn(bytes, *offset as usize + *addr_register as usize, from_register_list[i])),
                                    RegisterRead::Uniform(from_register) => (0..n).for_each(|_i| write_fn(bytes, *offset as usize + *addr_register as usize, *from_register)),
                                }
                            }
                        }
                    },
                    None => {
                        match from_register {
                            RegisterRead::Core(from_register_list) => (0..n).for_each(|i| write_fn(bytes, *offset as usize, from_register_list[i])),
                            RegisterRead::Uniform(from_register) => (0..n).for_each(|_i| write_fn(bytes, *offset as usize, *from_register)),
                        }
                    }
                }
            },
            ShaderInstruction::LoadTextureVector { src_xy_u32, load_type, dst, texture } => {
                let texture_number = resource_map.texture[*texture as usize] as usize;
                let texture = &texture_modules[texture_number];
                let pixel_layout = texture.config.pixel_layout;
                let coord_register = self.read_vector_register(*src_xy_u32, run_context);
                let vector_register = self.write_vector_register(*dst, run_context)?;
                let texture_load_op = match (*load_type, pixel_layout) {
                    (TextureLoadType::F32FromF32,  PixelDataLayout::D32x1)  |
                    (TextureLoadType::I32FromInt,  PixelDataLayout::D32x1)  |
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D32x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let x = texture.fetch::<u32>(u, v);
                        [x, 0, 0, 0]
                    },
                    (TextureLoadType::F32FromF32,  PixelDataLayout::D32x2)  |
                    (TextureLoadType::I32FromInt,  PixelDataLayout::D32x2)  |
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D32x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u32; 2]>(u, v);
                        [x, y, 0, 0]
                    },
                    (TextureLoadType::F32FromF32,  PixelDataLayout::D32x4)  |
                    (TextureLoadType::I32FromInt,  PixelDataLayout::D32x4)  |
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D32x4) => |u: u32, v: u32, texture: &TextureModule| texture.fetch::<[u32; 4]>(u, v),
                    (TextureLoadType::F32FromF32,  _                     ) => |_u: u32, _v: u32, _texture: &TextureModule| [0, 0, 0, 0],
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D16x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let x = texture.fetch::<u16>(u, v);
                        [x as u32, 0, 0, 0]
                    },
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D16x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u16; 2]>(u, v);
                        [x as u32, y as u32, 0, 0]
                    },
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D16x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u16; 4]>(u, v);
                        data.map(|x| x as u32)
                    },
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D8x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let x = texture.fetch::<u8>(u, v);
                        [x as u32, 0, 0, 0]
                    },
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D8x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u8; 2]>(u, v);
                        [x as u32, y as u32, 0, 0]
                    },
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D8x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u8; 4]>(u, v);
                        data.map(|x| x as u32)
                    },
                    (TextureLoadType::I32FromInt, PixelDataLayout::D16x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let x = texture.fetch::<u16>(u, v);
                        [x as i16 as i32 as u32, 0, 0, 0]
                    },
                    (TextureLoadType::I32FromInt, PixelDataLayout::D16x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u16; 2]>(u, v);
                        [x as i16 as i32 as u32, y as i16 as i32 as u32, 0, 0]
                    },
                    (TextureLoadType::I32FromInt, PixelDataLayout::D16x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u16; 4]>(u, v);
                        data.map(|x| x as i16 as i32 as u32)
                    },
                    (TextureLoadType::I32FromInt, PixelDataLayout::D8x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let x = texture.fetch::<u8>(u, v);
                        [x as i8 as i32 as u32, 0, 0, 0]
                    },
                    (TextureLoadType::I32FromInt, PixelDataLayout::D8x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u8; 2]>(u, v);
                        [x as i8 as i32 as u32, y as i8 as i32 as u32, 0, 0]
                    },
                    (TextureLoadType::I32FromInt, PixelDataLayout::D8x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u8; 4]>(u, v);
                        data.map(|x| x as i8 as i32 as u32)
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D32x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let x = texture.fetch::<u32>(u, v);
                        [(x as i32 as f32 / std::i32::MAX as f32).to_bits(), 0, 0, 0]
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D32x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u32; 2]>(u, v);
                        [(x as i32 as f32 / std::i32::MAX as f32).to_bits(), (y as i32 as f32 / std::i32::MAX as f32).to_bits(), 0, 0]
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D32x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u32; 4]>(u, v);
                        data.map(|x| (x as i32 as f32 / std::i32::MAX as f32).to_bits())
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D16x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let x = texture.fetch::<u16>(u, v);
                        [(x as i16 as f32 / std::i16::MAX as f32).to_bits(), 0, 0, 0]
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D16x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u16; 2]>(u, v);
                        [(x as i16 as f32 / std::i16::MAX as f32).to_bits(), (y as i16 as f32 / std::i16::MAX as f32).to_bits(), 0, 0]
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D16x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u16; 4]>(u, v);
                        data.map(|x| (x as i16 as f32 / std::i16::MAX as f32).to_bits())
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D8x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let x = texture.fetch::<u8>(u, v);
                        [(x as i8 as f32 / std::i8::MAX as f32).to_bits(), 0, 0, 0]
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D8x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u8; 2]>(u, v);
                        [(x as i8 as f32 / std::i8::MAX as f32).to_bits(), (y as i8 as f32 / std::i32::MAX as f32).to_bits(), 0, 0]
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D8x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u8; 4]>(u, v);
                        data.map(|x| (x as i8 as f32 / std::i8::MAX as f32).to_bits())
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D32x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let x = texture.fetch::<u32>(u, v);
                        [(x as u32 as f32 / std::u32::MAX as f32).to_bits(), 0, 0, 0]
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D32x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u32; 2]>(u, v);
                        [(x as u32 as f32 / std::u32::MAX as f32).to_bits(), (y as u32 as f32 / std::u32::MAX as f32).to_bits(), 0, 0]
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D32x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u32; 4]>(u, v);
                        data.map(|x| (x as u32 as f32 / std::u32::MAX as f32).to_bits())
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D16x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let x = texture.fetch::<u16>(u, v);
                        [(x as u16 as f32 / std::u16::MAX as f32).to_bits(), 0, 0, 0]
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D16x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u16; 2]>(u, v);
                        [(x as u16 as f32 / std::u16::MAX as f32).to_bits(), (y as u16 as f32 / std::u16::MAX as f32).to_bits(), 0, 0]
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D16x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u16; 4]>(u, v);
                        data.map(|x| (x as u16 as f32 / std::u16::MAX as f32).to_bits())
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D8x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let x = texture.fetch::<u8>(u, v);
                        [(x as u8 as f32 / std::u8::MAX as f32).to_bits(), 0, 0, 0]
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D8x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u8; 2]>(u, v);
                        [(x as u8 as f32 / std::u8::MAX as f32).to_bits(), (y as u8 as f32 / std::u32::MAX as f32).to_bits(), 0, 0]
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D8x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u8; 4]>(u, v);
                        data.map(|x| (x as u8 as f32 / std::u8::MAX as f32).to_bits())
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D32x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let x = texture.fetch::<u32>(u, v);
                        [(x as i32 as f32).to_bits(), 0, 0, 0]
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D32x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u32; 2]>(u, v);
                        [(x as i32 as f32).to_bits(), (y as i32 as f32).to_bits(), 0, 0]
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D32x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u32; 4]>(u, v);
                        data.map(|x| (x as i32 as f32).to_bits())
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D16x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let x = texture.fetch::<u16>(u, v);
                        [(x as i16 as f32).to_bits(), 0, 0, 0]
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D16x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u16; 2]>(u, v);
                        [(x as i16 as f32).to_bits(), (y as i32 as f32).to_bits(), 0, 0]
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D16x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u16; 4]>(u, v);
                        data.map(|x| (x as i16 as f32).to_bits())
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D8x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let x = texture.fetch::<u8>(u, v);
                        [(x as i8 as f32).to_bits(), 0, 0, 0]
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D8x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u8; 2]>(u, v);
                        [(x as i8 as f32).to_bits(), (y as i32 as f32).to_bits(), 0, 0]
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D8x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u8; 4]>(u, v);
                        data.map(|x| (x as i8 as f32).to_bits())
                    },

                    (TextureLoadType::F32FromUInt, PixelDataLayout::D32x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<u32>(u, v);
                        [(data as f32).to_bits(), 0, 0, 0]
                    },
                    (TextureLoadType::F32FromUInt, PixelDataLayout::D32x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u32; 2]>(u, v);
                        [(x as f32).to_bits(), (y as f32).to_bits(), 0, 0]
                    },
                    (TextureLoadType::F32FromUInt, PixelDataLayout::D32x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u32; 4]>(u, v);
                        data.map(|x| (x as f32).to_bits())
                    },
                    (TextureLoadType::F32FromUInt, PixelDataLayout::D16x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<u16>(u, v);
                        [(data as f32).to_bits(), 0, 0, 0]
                    },
                    (TextureLoadType::F32FromUInt, PixelDataLayout::D16x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u16; 2]>(u, v);
                        [(x as f32).to_bits(), (y as f32).to_bits(), 0, 0]
                    },
                    (TextureLoadType::F32FromUInt, PixelDataLayout::D16x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u16; 4]>(u, v);
                        data.map(|x| (x as f32).to_bits())
                    },
                    (TextureLoadType::F32FromUInt, PixelDataLayout::D8x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<u8>(u, v);
                        [(data as f32).to_bits(), 0, 0, 0]
                    },
                    (TextureLoadType::F32FromUInt, PixelDataLayout::D8x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u8; 2]>(u, v);
                        [(x as f32).to_bits(), (y as f32).to_bits(), 0, 0]
                    },
                    (TextureLoadType::F32FromUInt, PixelDataLayout::D8x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u8; 4]>(u, v);
                        data.map(|x| (x as f32).to_bits())
                    },
                    (TextureLoadType::I32FromF32, PixelDataLayout::D32x1) => |u: u32, v: u32, texture: &TextureModule| {
                        let x = texture.fetch::<u32>(u, v);
                        [f32::from_bits(x) as i32 as u32, 0, 0, 0]
                    },
                    (TextureLoadType::I32FromF32, PixelDataLayout::D32x2) => |u: u32, v: u32, texture: &TextureModule| {
                        let [x, y] = texture.fetch::<[u32; 2]>(u, v);
                        [f32::from_bits(x) as i32 as u32, f32::from_bits(y) as i32 as u32, 0, 0]
                    },
                    (TextureLoadType::I32FromF32, PixelDataLayout::D32x4) => |u: u32, v: u32, texture: &TextureModule| {
                        let data = texture.fetch::<[u32; 4]>(u, v);
                        data.map(|x| f32::from_bits(x) as i32 as u32)
                    },
                    (TextureLoadType::I32FromF32, _) => |_u: u32, _v: u32, _texture: &TextureModule| [0, 0, 0, 0],
                };
                match coord_register {
                    RegisterRead::Core(coord_register_list) => (0..n).for_each(|i| {
                        let [x, y, ..] = coord_register_list[i];
                        vector_register[i] = texture_load_op(x, y, texture);
                    }),
                    RegisterRead::Uniform(coord_register) => {
                        let [x, y, ..] = *coord_register;
                        let value = texture_load_op(x, y, texture);
                        vector_register[0..n].fill(value);
                    },
                }
            },
            ShaderInstruction::LoadTextureScalar { src_xy_u32, load_type, channel, dst, texture } => {
                let texture_number = resource_map.texture[*texture as usize] as usize;
                let texture = &texture_modules[texture_number];
                let pixel_layout = texture.config.pixel_layout;
                let scalar_register = self.write_scalar_register(*dst, run_context)?;
                let coord_register = self.read_vector_register(*src_xy_u32, run_context);
                match (pixel_layout, channel) {
                    (PixelDataLayout::D8x1,  VectorChannel::Y) |
                    (PixelDataLayout::D8x1,  VectorChannel::Z) |
                    (PixelDataLayout::D8x1,  VectorChannel::W) |
                    (PixelDataLayout::D16x1, VectorChannel::Y) |
                    (PixelDataLayout::D16x1, VectorChannel::Z) |
                    (PixelDataLayout::D16x1, VectorChannel::W) |
                    (PixelDataLayout::D32x1, VectorChannel::Y) |
                    (PixelDataLayout::D32x1, VectorChannel::Z) |
                    (PixelDataLayout::D32x1, VectorChannel::W) |
                    (PixelDataLayout::D8x2,  VectorChannel::Z) |
                    (PixelDataLayout::D8x2,  VectorChannel::W) |
                    (PixelDataLayout::D16x2, VectorChannel::Z) |
                    (PixelDataLayout::D16x2, VectorChannel::W) |
                    (PixelDataLayout::D32x2, VectorChannel::Z) |
                    (PixelDataLayout::D32x2, VectorChannel::W) => None?,
                    _ => {}
                };
                let channel = *channel as usize;
                let texture_load_op = match (*load_type, pixel_layout) {
                    (TextureLoadType::F32FromF32,  PixelDataLayout::D8x1 ) |
                    (TextureLoadType::F32FromF32,  PixelDataLayout::D8x2 ) |
                    (TextureLoadType::F32FromF32,  PixelDataLayout::D8x4 ) |
                    (TextureLoadType::F32FromF32,  PixelDataLayout::D16x1) |
                    (TextureLoadType::F32FromF32,  PixelDataLayout::D16x2) |
                    (TextureLoadType::F32FromF32,  PixelDataLayout::D16x4) |
                    (TextureLoadType::I32FromF32,  PixelDataLayout::D8x1 ) |
                    (TextureLoadType::I32FromF32,  PixelDataLayout::D8x2 ) |
                    (TextureLoadType::I32FromF32,  PixelDataLayout::D8x4 ) |
                    (TextureLoadType::I32FromF32,  PixelDataLayout::D16x1) |
                    (TextureLoadType::I32FromF32,  PixelDataLayout::D16x2) |
                    (TextureLoadType::I32FromF32,  PixelDataLayout::D16x4) => None?,

                    (TextureLoadType::I32FromF32,  PixelDataLayout::D32x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u32>(u, v);
                        f32::from_bits(x) as i32 as u32
                    },
                    (TextureLoadType::I32FromF32,  PixelDataLayout::D32x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u32; 2]>(u, v);
                        f32::from_bits(data[channel]) as i32 as u32
                    },
                    (TextureLoadType::I32FromF32,  PixelDataLayout::D32x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u32; 2]>(u, v);
                        f32::from_bits(data[channel]) as i32 as u32
                    },

                    (TextureLoadType::I32FromInt,  PixelDataLayout::D32x1) |
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D32x1) |
                    (TextureLoadType::F32FromF32,  PixelDataLayout::D32x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u32>(u, v);
                        x
                    },
                    (TextureLoadType::I32FromInt,  PixelDataLayout::D32x2) |
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D32x2) |
                    (TextureLoadType::F32FromF32,  PixelDataLayout::D32x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u32; 2]>(u, v);
                        data[channel]
                    },
                    (TextureLoadType::I32FromInt,  PixelDataLayout::D32x4) |
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D32x4) |
                    (TextureLoadType::F32FromF32,  PixelDataLayout::D32x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u32; 4]>(u, v);
                        data[channel]
                    },
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D16x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u16>(u, v);
                        x as u32
                    },
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D16x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u16; 2]>(u, v);
                        data[channel] as u32
                    },
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D16x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u16; 4]>(u, v);
                        data[channel] as u32
                    },
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D8x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u8>(u, v);
                        x as u32
                    },
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D8x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u8; 2]>(u, v);
                        data[channel] as u32
                    },
                    (TextureLoadType::I32FromUInt, PixelDataLayout::D8x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u8; 4]>(u, v);
                        data[channel] as u32
                    },
                    (TextureLoadType::I32FromInt,  PixelDataLayout::D16x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u16>(u, v);
                        x as i16 as i32 as u32
                    },
                    (TextureLoadType::I32FromInt,  PixelDataLayout::D16x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u16; 2]>(u, v);
                        data[channel] as i16 as i32 as u32
                    },
                    (TextureLoadType::I32FromInt,  PixelDataLayout::D16x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u16; 4]>(u, v);
                        data[channel] as i16 as i32 as u32
                    },
                    (TextureLoadType::I32FromInt,  PixelDataLayout::D8x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u8>(u, v);
                        x as i8 as i32 as u32
                    },
                    (TextureLoadType::I32FromInt,  PixelDataLayout::D8x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u8; 2]>(u, v);
                        data[channel] as i8 as i32 as u32
                    },
                    (TextureLoadType::I32FromInt,  PixelDataLayout::D8x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u8; 4]>(u, v);
                        data[channel] as i8 as i32 as u32
                    },

                    (TextureLoadType::F32FromINorm, PixelDataLayout::D32x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u32>(u, v);
                        (x as i32 as f32 / std::i32::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D32x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u32; 2]>(u, v);
                        (data[channel] as i32 as f32 / std::i32::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D32x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u32; 4]>(u, v);
                        (data[channel] as i32 as f32 / std::i32::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D16x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u16>(u, v);
                        (x as i16 as i32 as f32 / std::i16::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D16x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u16; 2]>(u, v);
                        (data[channel] as i16 as i32 as f32 / std::i16::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D16x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u16; 4]>(u, v);
                        (data[channel] as i16 as i32 as f32 / std::i16::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D8x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u8>(u, v);
                        (x as i8 as i32 as f32 / std::i8::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D8x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u8; 2]>(u, v);
                        (data[channel] as i8 as i32 as f32 / std::i8::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromINorm, PixelDataLayout::D8x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u8; 4]>(u, v);
                        (data[channel] as i8 as i32 as f32 / std::i8::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D32x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u32>(u, v);
                        (x as f32 / std::u32::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D32x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u32; 2]>(u, v);
                        (data[channel] as f32 / std::u32::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D32x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u32; 4]>(u, v);
                        (data[channel] as f32 / std::u32::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D16x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u16>(u, v);
                        (x as f32 / std::u16::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D16x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u16; 2]>(u, v);
                        (data[channel] as f32 / std::u16::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D16x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u16; 4]>(u, v);
                        (data[channel] as f32 / std::u16::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D8x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u8>(u, v);
                        (x as f32 / std::u8::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D8x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u8; 2]>(u, v);
                        (data[channel] as f32 / std::u8::MAX as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUNorm, PixelDataLayout::D8x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u8; 4]>(u, v);
                        (data[channel] as f32 / std::u8::MAX as f32).to_bits()
                    },

                    (TextureLoadType::F32FromInt, PixelDataLayout::D32x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u32>(u, v);
                        (x as i32 as f32).to_bits()
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D32x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u32; 2]>(u, v);
                        (data[channel] as i32 as f32).to_bits()
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D32x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u32; 4]>(u, v);
                        (data[channel] as i32 as f32).to_bits()
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D16x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u16>(u, v);
                        (x as i16 as f32).to_bits()
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D16x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u16; 2]>(u, v);
                        (data[channel] as i16 as f32).to_bits()
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D16x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u16; 4]>(u, v);
                        (data[channel] as i16 as f32).to_bits()
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D8x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u8>(u, v);
                        (x as i8 as f32).to_bits()
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D8x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u8; 2]>(u, v);
                        (data[channel] as i8 as f32).to_bits()
                    },
                    (TextureLoadType::F32FromInt, PixelDataLayout::D8x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u8; 4]>(u, v);
                        (data[channel] as i8 as f32).to_bits()
                    },

                    (TextureLoadType::F32FromUInt, PixelDataLayout::D32x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u32>(u, v);
                        (x as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUInt, PixelDataLayout::D32x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u32; 2]>(u, v);
                        (data[channel] as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUInt, PixelDataLayout::D32x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u32; 4]>(u, v);
                        (data[channel] as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUInt, PixelDataLayout::D16x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u16>(u, v);
                        (x as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUInt, PixelDataLayout::D16x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u16; 2]>(u, v);
                        (data[channel] as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUInt, PixelDataLayout::D16x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u16; 4]>(u, v);
                        (data[channel] as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUInt, PixelDataLayout::D8x1) => |u: u32, v: u32, texture: &TextureModule, _channel: usize| {
                        let x = texture.fetch::<u8>(u, v);
                        (x as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUInt, PixelDataLayout::D8x2) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u8; 2]>(u, v);
                        (data[channel] as f32).to_bits()
                    },
                    (TextureLoadType::F32FromUInt, PixelDataLayout::D8x4) => |u: u32, v: u32, texture: &TextureModule, channel: usize| {
                        let data = texture.fetch::<[u8; 4]>(u, v);
                        (data[channel] as f32).to_bits()
                    },
                };
                match coord_register {
                    RegisterRead::Core(coord_register_list) => (0..n).for_each(|i| {
                        let [x, y, ..] = coord_register_list[i];
                        scalar_register[i] = texture_load_op(x, y, texture, channel);
                    }),
                    RegisterRead::Uniform(coord_register) => {
                        let [x, y, ..] = *coord_register;
                        let value = texture_load_op(x, y, texture, channel);
                        scalar_register[0..n].fill(value);
                    },
                }
            },
            ShaderInstruction::ScalarUnaryOp { src, dst, op } => {
                let from_register = self.read_scalar_register(*src, run_context);
                let to_register = self.write_scalar_register(*dst, run_context)?;
                let op_fn = match op {
                    ScalarUnaryOp::Convert(OpDataTypeConversion::F32ToI32) => |x: u32| f32::from_bits(x) as i32 as u32,
                    ScalarUnaryOp::Convert(OpDataTypeConversion::F32ToU32) => |x: u32| f32::from_bits(x) as u32,
                    ScalarUnaryOp::Convert(OpDataTypeConversion::U32ToF32) => |x: u32| (x as f32).to_bits(),
                    ScalarUnaryOp::Convert(OpDataTypeConversion::I32toF32) => |x: u32| (x as i32 as f32).to_bits(),
                    ScalarUnaryOp::Negative(OpDataType::F32)               => |x: u32| (- f32::from_bits(x)).to_bits(),
                    ScalarUnaryOp::Negative(OpDataType::I32)               => |x: u32| (- (x as i32)) as u32,
                    ScalarUnaryOp::Sign(OpDataType::F32)                   => |x: u32| f32::from_bits(x).signum().to_bits(),
                    ScalarUnaryOp::Sign(OpDataType::I32)                   => |x: u32| (x as i32).signum() as u32,
                    ScalarUnaryOp::Reciporocal                             => |x: u32| f32::from_bits(x).recip().to_bits(),
                    ScalarUnaryOp::Sin                                     => |x: u32| f32::from_bits(x).sin().to_bits(),
                    ScalarUnaryOp::Cos                                     => |x: u32| f32::from_bits(x).cos().to_bits(),
                    ScalarUnaryOp::Tan                                     => |x: u32| f32::from_bits(x).tan().to_bits(),
                    ScalarUnaryOp::ASin                                    => |x: u32| f32::from_bits(x).asin().to_bits(),
                    ScalarUnaryOp::ACos                                    => |x: u32| f32::from_bits(x).acos().to_bits(),
                    ScalarUnaryOp::Atan                                    => |x: u32| f32::from_bits(x).atan().to_bits(),
                    ScalarUnaryOp::Ln                                      => |x: u32| f32::from_bits(x).ln().to_bits(),
                    ScalarUnaryOp::Exp                                     => |x: u32| f32::from_bits(x).exp().to_bits(),
                };
                match from_register {
                    RegisterRead::Core(from_register_list) => (0..n).for_each(|i| to_register[i] = op_fn(from_register_list[i])),
                    RegisterRead::Uniform(from_register) => to_register[0..n].fill(op_fn(*from_register)),
                }
            },
            ShaderInstruction::ScalarBinaryOp { src_a, src_b, dst, op } => {
                let from_a_register = self.read_scalar_register(*src_a, run_context);
                let from_b_register = self.read_scalar_register(*src_b, run_context);
                let to_register = self.write_scalar_register(*dst, run_context)?;
                let op_fn = match op {
                    ScalarBinaryOp::Compare(OpDataType::F32, Comparison::Equal)           => |a: u32, b: u32| if f32::from_bits(a) == f32::from_bits(b) { 1 } else { 0 },
                    ScalarBinaryOp::Compare(OpDataType::F32, Comparison::NotEqual)        => |a: u32, b: u32| if f32::from_bits(a) != f32::from_bits(b) { 1 } else { 0 },
                    ScalarBinaryOp::Compare(OpDataType::F32, Comparison::LessThanOrEqual) => |a: u32, b: u32| if f32::from_bits(a) <= f32::from_bits(b) { 1 } else { 0 },
                    ScalarBinaryOp::Compare(OpDataType::F32, Comparison::GreaterThan)     => |a: u32, b: u32| if f32::from_bits(a) >  f32::from_bits(b) { 1 } else { 0 },
                    ScalarBinaryOp::Compare(OpDataType::I32, Comparison::Equal)           => |a: u32, b: u32| if a as i32 == b as i32 { 1 } else { 0 },
                    ScalarBinaryOp::Compare(OpDataType::I32, Comparison::NotEqual)        => |a: u32, b: u32| if a as i32 != b as i32 { 1 } else { 0 },
                    ScalarBinaryOp::Compare(OpDataType::I32, Comparison::LessThanOrEqual) => |a: u32, b: u32| if a as i32 <= b as i32 { 1 } else { 0 },
                    ScalarBinaryOp::Compare(OpDataType::I32, Comparison::GreaterThan)     => |a: u32, b: u32| if a as i32 >  b as i32 { 1 } else { 0 },
                    ScalarBinaryOp::CompareUnsigned(Comparison::Equal)                    => |a: u32, b: u32| if a == b { 1 } else { 0 },
                    ScalarBinaryOp::CompareUnsigned(Comparison::NotEqual)                 => |a: u32, b: u32| if a != b { 1 } else { 0 },
                    ScalarBinaryOp::CompareUnsigned(Comparison::LessThanOrEqual)          => |a: u32, b: u32| if a <= b { 1 } else { 0 },
                    ScalarBinaryOp::CompareUnsigned(Comparison::GreaterThan)              => |a: u32, b: u32| if a >  b { 1 } else { 0 },
                    ScalarBinaryOp::And                                                   => |a: u32, b: u32| a & b,
                    ScalarBinaryOp::Or                                                    => |a: u32, b: u32| a | b,
                    ScalarBinaryOp::Xor                                                   => |a: u32, b: u32| a ^ b,
                    ScalarBinaryOp::AndNot                                                => |a: u32, b: u32| a & !b,
                    ScalarBinaryOp::Add(OpDataType::I32)                                  => |a: u32, b: u32| (a as i32).wrapping_add(b as i32) as u32,
                    ScalarBinaryOp::Add(OpDataType::F32)                                  => |a: u32, b: u32| (f32::from_bits(a) + f32::from_bits(b)).to_bits(),
                    ScalarBinaryOp::Subtract(OpDataType::I32)                             => |a: u32, b: u32| (a as i32).wrapping_sub(b as i32) as u32,
                    ScalarBinaryOp::Subtract(OpDataType::F32)                             => |a: u32, b: u32| (f32::from_bits(a) + f32::from_bits(b)).to_bits(),
                    ScalarBinaryOp::Multiply(OpDataType::I32)                             => |a: u32, b: u32| (a as i32).wrapping_mul(b as i32) as u32,
                    ScalarBinaryOp::Multiply(OpDataType::F32)                             => |a: u32, b: u32| (f32::from_bits(a) * f32::from_bits(b)).to_bits(),
                    ScalarBinaryOp::Divide(OpDataType::I32)                               => |a: u32, b: u32| (a as i32).wrapping_div(b as i32) as u32,
                    ScalarBinaryOp::Divide(OpDataType::F32)                               => |a: u32, b: u32| (f32::from_bits(a) / f32::from_bits(b)).to_bits(),
                    ScalarBinaryOp::Modulo(OpDataType::I32)                               => |a: u32, b: u32| (a as i32).rem(b as i32) as u32,
                    ScalarBinaryOp::Modulo(OpDataType::F32)                               => |a: u32, b: u32| f32::from_bits(a).rem(f32::from_bits(b)).to_bits(),
                    ScalarBinaryOp::Atan2                                                 => |a: u32, b: u32| f32::atan2(f32::from_bits(a), f32::from_bits(b)).to_bits(),
                };
                match (from_a_register, from_b_register) {
                    (RegisterRead::Core(from_a_reg_list), RegisterRead::Core(from_b_reg_list)) => 
                        (0..n).for_each(|i| to_register[i] = op_fn(from_a_reg_list[i], from_b_reg_list[i])),
                    (RegisterRead::Core(from_a_reg_list), RegisterRead::Uniform(from_b_reg)  ) => 
                        (0..n).for_each(|i| to_register[i] = op_fn(from_a_reg_list[i], *from_b_reg)),
                    (RegisterRead::Uniform(from_a_reg),  RegisterRead::Core(from_b_reg_list)) => 
                        (0..n).for_each(|i| to_register[i] = op_fn(*from_a_reg, from_b_reg_list[i])),
                    (RegisterRead::Uniform(from_a_reg),  RegisterRead::Uniform(from_b_reg)  ) => 
                        to_register[0..n].fill(op_fn(*from_a_reg, *from_b_reg)),
                }
            },
            ShaderInstruction::VectorComponentwiseScalarUnaryOp { src, dst, op } => {
                let from_register = self.read_vector_register(*src, run_context);
                let to_register = self.write_vector_register(*dst, run_context)?;
                let op_fn = match op {
                    ScalarUnaryOp::Convert(OpDataTypeConversion::F32ToI32) => |x: [u32; 4]| [0, 1, 2, 3].map(|c| f32::from_bits(x[c]) as i32 as u32),
                    ScalarUnaryOp::Convert(OpDataTypeConversion::F32ToU32) => |x: [u32; 4]| [0, 1, 2, 3].map(|c| f32::from_bits(x[c]) as u32),
                    ScalarUnaryOp::Convert(OpDataTypeConversion::U32ToF32) => |x: [u32; 4]| [0, 1, 2, 3].map(|c| (x[c] as f32).to_bits()),
                    ScalarUnaryOp::Convert(OpDataTypeConversion::I32toF32) => |x: [u32; 4]| [0, 1, 2, 3].map(|c| (x[c] as i32 as f32).to_bits()),
                    ScalarUnaryOp::Negative(OpDataType::F32)               => |x: [u32; 4]| [0, 1, 2, 3].map(|c| (- f32::from_bits(x[c])).to_bits()),
                    ScalarUnaryOp::Negative(OpDataType::I32)               => |x: [u32; 4]| [0, 1, 2, 3].map(|c| (- (x[c] as i32)) as u32),
                    ScalarUnaryOp::Sign(OpDataType::F32)                   => |x: [u32; 4]| [0, 1, 2, 3].map(|c| f32::from_bits(x[c]).signum().to_bits()),
                    ScalarUnaryOp::Sign(OpDataType::I32)                   => |x: [u32; 4]| [0, 1, 2, 3].map(|c| (x[c] as i32).signum() as u32),
                    ScalarUnaryOp::Reciporocal                             => |x: [u32; 4]| [0, 1, 2, 3].map(|c| f32::from_bits(x[c]).recip().to_bits()),
                    ScalarUnaryOp::Sin                                     => |x: [u32; 4]| [0, 1, 2, 3].map(|c| f32::from_bits(x[c]).sin().to_bits()),
                    ScalarUnaryOp::Cos                                     => |x: [u32; 4]| [0, 1, 2, 3].map(|c| f32::from_bits(x[c]).cos().to_bits()),
                    ScalarUnaryOp::Tan                                     => |x: [u32; 4]| [0, 1, 2, 3].map(|c| f32::from_bits(x[c]).tan().to_bits()),
                    ScalarUnaryOp::ASin                                    => |x: [u32; 4]| [0, 1, 2, 3].map(|c| f32::from_bits(x[c]).asin().to_bits()),
                    ScalarUnaryOp::ACos                                    => |x: [u32; 4]| [0, 1, 2, 3].map(|c| f32::from_bits(x[c]).acos().to_bits()),
                    ScalarUnaryOp::Atan                                    => |x: [u32; 4]| [0, 1, 2, 3].map(|c| f32::from_bits(x[c]).atan().to_bits()),
                    ScalarUnaryOp::Ln                                      => |x: [u32; 4]| [0, 1, 2, 3].map(|c| f32::from_bits(x[c]).ln().to_bits()),
                    ScalarUnaryOp::Exp                                     => |x: [u32; 4]| [0, 1, 2, 3].map(|c| f32::from_bits(x[c]).exp().to_bits()),
                };
                match from_register {
                    RegisterRead::Core(from_register_list) => (0..n).for_each(|i| to_register[i] = op_fn(from_register_list[i])),
                    RegisterRead::Uniform(from_register) => to_register[0..n].fill(op_fn(*from_register)),
                }
            },
            ShaderInstruction::VectorComponentwiseScalarBinaryOp { src_a, src_b, dst, op } => {
                let from_a_register = self.read_vector_register(*src_a, run_context);
                let from_b_register = self.read_vector_register(*src_b, run_context);
                let to_register = self.write_vector_register(*dst, run_context)?;
                let op_fn = match op {
                    ScalarBinaryOp::Compare(OpDataType::F32, Comparison::Equal)           => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| if f32::from_bits(a[c]) == f32::from_bits(b[c]) { 1 } else { 0 }),
                    ScalarBinaryOp::Compare(OpDataType::F32, Comparison::NotEqual)        => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| if f32::from_bits(a[c]) != f32::from_bits(b[c]) { 1 } else { 0 }),
                    ScalarBinaryOp::Compare(OpDataType::F32, Comparison::LessThanOrEqual) => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| if f32::from_bits(a[c]) <= f32::from_bits(b[c]) { 1 } else { 0 }),
                    ScalarBinaryOp::Compare(OpDataType::F32, Comparison::GreaterThan)     => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| if f32::from_bits(a[c]) >  f32::from_bits(b[c]) { 1 } else { 0 }),
                    ScalarBinaryOp::Compare(OpDataType::I32, Comparison::Equal)           => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| if a[c] as i32 == b[c] as i32 { 1 } else { 0 }),
                    ScalarBinaryOp::Compare(OpDataType::I32, Comparison::NotEqual)        => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| if a[c] as i32 != b[c] as i32 { 1 } else { 0 }),
                    ScalarBinaryOp::Compare(OpDataType::I32, Comparison::LessThanOrEqual) => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| if a[c] as i32 <= b[c] as i32 { 1 } else { 0 }),
                    ScalarBinaryOp::Compare(OpDataType::I32, Comparison::GreaterThan)     => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| if a[c] as i32 >  b[c] as i32 { 1 } else { 0 }),
                    ScalarBinaryOp::CompareUnsigned(Comparison::Equal)                    => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| if a[c] == b[c] { 1 } else { 0 }),
                    ScalarBinaryOp::CompareUnsigned(Comparison::NotEqual)                 => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| if a[c] != b[c] { 1 } else { 0 }),
                    ScalarBinaryOp::CompareUnsigned(Comparison::LessThanOrEqual)          => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| if a[c] <= b[c] { 1 } else { 0 }),
                    ScalarBinaryOp::CompareUnsigned(Comparison::GreaterThan)              => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| if a[c] >  b[c] { 1 } else { 0 }),
                    ScalarBinaryOp::And                                                   => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| a[c] & b[c]),
                    ScalarBinaryOp::AndNot                                                => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| a[c] & !b[c]),
                    ScalarBinaryOp::Or                                                    => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| a[c] | b[c]),
                    ScalarBinaryOp::Xor                                                   => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| a[c] ^ b[c]),
                    ScalarBinaryOp::Add(OpDataType::I32)                                  => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| (a[c] as i32).wrapping_add(b[c] as i32) as u32),
                    ScalarBinaryOp::Add(OpDataType::F32)                                  => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| (f32::from_bits(a[c]) + f32::from_bits(b[c])).to_bits()),
                    ScalarBinaryOp::Subtract(OpDataType::I32)                             => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| (a[c] as i32).wrapping_sub(b[c] as i32) as u32),
                    ScalarBinaryOp::Subtract(OpDataType::F32)                             => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| (f32::from_bits(a[c]) + f32::from_bits(b[c])).to_bits()),
                    ScalarBinaryOp::Multiply(OpDataType::I32)                             => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| (a[c] as i32).wrapping_mul(b[c] as i32) as u32),
                    ScalarBinaryOp::Multiply(OpDataType::F32)                             => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| (f32::from_bits(a[c]) * f32::from_bits(b[c])).to_bits()),
                    ScalarBinaryOp::Divide(OpDataType::I32)                               => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| (a[c] as i32).wrapping_div(b[c] as i32) as u32),
                    ScalarBinaryOp::Divide(OpDataType::F32)                               => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| (f32::from_bits(a[c]) / f32::from_bits(b[c])).to_bits()),
                    ScalarBinaryOp::Modulo(OpDataType::I32)                               => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| (a[c] as i32).rem(b[c] as i32) as u32),
                    ScalarBinaryOp::Modulo(OpDataType::F32)                               => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| f32::from_bits(a[c]).rem(f32::from_bits(b[c])).to_bits()),
                    ScalarBinaryOp::Atan2                                                 => |a: [u32; 4], b: [u32; 4]| [0, 1, 2, 3].map(|c| f32::atan2(f32::from_bits(a[c]), f32::from_bits(b[c])).to_bits()),
                };
                match (from_a_register, from_b_register) {
                    (RegisterRead::Core(from_a_reg_list), RegisterRead::Core(from_b_reg_list)) => 
                        (0..n).for_each(|i| to_register[i] = op_fn(from_a_reg_list[i], from_b_reg_list[i])),
                    (RegisterRead::Core(from_a_reg_list), RegisterRead::Uniform(from_b_reg)  ) => 
                        (0..n).for_each(|i| to_register[i] = op_fn(from_a_reg_list[i], *from_b_reg)),
                    (RegisterRead::Uniform(from_a_reg),  RegisterRead::Core(from_b_reg_list)) => 
                        (0..n).for_each(|i| to_register[i] = op_fn(*from_a_reg, from_b_reg_list[i])),
                    (RegisterRead::Uniform(from_a_reg),  RegisterRead::Uniform(from_b_reg)  ) => 
                        to_register[0..n].fill(op_fn(*from_a_reg, *from_b_reg)),
                }
            },
            ShaderInstruction::VectorComponentwiseScalarTernaryOp { src_a, src_b, src_c, dst, op } => {
                let from_a_register = self.read_vector_register(*src_a, run_context);
                let from_b_register = self.read_vector_register(*src_b, run_context);
                let from_c_register = self.read_vector_register(*src_c, run_context);
                let to_register = self.write_vector_register(*dst, run_context)?;
                let op_fn = match op {
                    ScalarTernaryOp::Fma(OpDataType::F32) => |a: [u32; 4], b: [u32; 4], c: [u32; 4]| [0, 1, 2, 3].map(|i| {
                        (f32::from_bits(a[i]) * f32::from_bits(b[i]) + f32::from_bits(c[i])).to_bits()
                    }),
                    ScalarTernaryOp::Fma(OpDataType::I32) => |a: [u32; 4], b: [u32; 4], c: [u32; 4]| [0, 1, 2, 3].map(|i| {
                        (a[i] as i32).wrapping_mul(b[i] as i32).wrapping_add(c[i] as i32) as u32
                    }),
                };
                for i in 0..n {
                    let from_a = match from_a_register {
                        RegisterRead::Core(register_list) => register_list[i],
                        RegisterRead::Uniform(register) => *register
                    };
                    let from_b = match from_b_register {
                        RegisterRead::Core(register_list) => register_list[i],
                        RegisterRead::Uniform(register) => *register
                    };
                    let from_c = match from_c_register {
                        RegisterRead::Core(register_list) => register_list[i],
                        RegisterRead::Uniform(register) => *register
                    };
                    to_register[i] = op_fn(from_a, from_b, from_c);
                }
            },
            ShaderInstruction::VectorComponentwiseScalarUnaryOpMasked { src, dst, op, mask } => {
                let from_register = self.read_vector_register(*src, run_context);
                let to_register = self.write_vector_register(*dst, run_context)?;
                let op_fn = match op {
                    ScalarUnaryOp::Convert(OpDataTypeConversion::F32ToI32) => |x: u32| f32::from_bits(x) as i32 as u32,
                    ScalarUnaryOp::Convert(OpDataTypeConversion::F32ToU32) => |x: u32| f32::from_bits(x) as u32,
                    ScalarUnaryOp::Convert(OpDataTypeConversion::U32ToF32) => |x: u32| (x as f32).to_bits(),
                    ScalarUnaryOp::Convert(OpDataTypeConversion::I32toF32) => |x: u32| (x as i32 as f32).to_bits(),
                    ScalarUnaryOp::Negative(OpDataType::F32)               => |x: u32| (- f32::from_bits(x)).to_bits(),
                    ScalarUnaryOp::Negative(OpDataType::I32)               => |x: u32| (- (x as i32)) as u32,
                    ScalarUnaryOp::Sign(OpDataType::F32)                   => |x: u32| f32::from_bits(x).signum().to_bits(),
                    ScalarUnaryOp::Sign(OpDataType::I32)                   => |x: u32| (x as i32).signum() as u32,
                    ScalarUnaryOp::Reciporocal                             => |x: u32| f32::from_bits(x).recip().to_bits(),
                    ScalarUnaryOp::Sin                                     => |x: u32| f32::from_bits(x).sin().to_bits(),
                    ScalarUnaryOp::Cos                                     => |x: u32| f32::from_bits(x).cos().to_bits(),
                    ScalarUnaryOp::Tan                                     => |x: u32| f32::from_bits(x).tan().to_bits(),
                    ScalarUnaryOp::ASin                                    => |x: u32| f32::from_bits(x).asin().to_bits(),
                    ScalarUnaryOp::ACos                                    => |x: u32| f32::from_bits(x).acos().to_bits(),
                    ScalarUnaryOp::Atan                                    => |x: u32| f32::from_bits(x).atan().to_bits(),
                    ScalarUnaryOp::Ln                                      => |x: u32| f32::from_bits(x).ln().to_bits(),
                    ScalarUnaryOp::Exp                                     => |x: u32| f32::from_bits(x).exp().to_bits(),
                };
                for c in 0..4 {
                    if (*mask & (1 << c)) != 0 {
                        for i in 0..n {
                            let from = match from_register {
                                RegisterRead::Core(register_list) => (register_list[i])[c],
                                RegisterRead::Uniform(register) => (*register)[c]
                            };
                            (to_register[i])[c] = op_fn(from);
                        }
                    }
                }
            },
            ShaderInstruction::MatrixMultiply4x4V4 { a0, a1, a2, a3, x, dest } => {
                let from_a0_reg = self.read_vector_register(*a0, run_context);
                let from_a1_reg = self.read_vector_register(*a1, run_context);
                let from_a2_reg = self.read_vector_register(*a2, run_context);
                let from_a3_reg = self.read_vector_register(*a3, run_context);
                let x_reg = self.read_vector_register(*x, run_context);
                let dest_reg = self.write_vector_register(*dest, run_context)?;
                for i in 0..n {
                    let a0 = match from_a0_reg {
                        RegisterRead::Core(reg_list) => reg_list[i],
                        RegisterRead::Uniform(reg) => *reg
                    }.map(|x| f32::from_bits(x));
                    let a1 = match from_a1_reg {
                        RegisterRead::Core(reg_list) => reg_list[i],
                        RegisterRead::Uniform(reg) => *reg
                    }.map(|x| f32::from_bits(x));
                    let a2 = match from_a2_reg {
                        RegisterRead::Core(reg_list) => reg_list[i],
                        RegisterRead::Uniform(reg) => *reg
                    }.map(|x| f32::from_bits(x));
                    let a3 = match from_a3_reg {
                        RegisterRead::Core(reg_list) => reg_list[i],
                        RegisterRead::Uniform(reg) => *reg
                    }.map(|x| f32::from_bits(x));
                    let x = match x_reg {
                        RegisterRead::Core(reg_list) => reg_list[i],
                        RegisterRead::Uniform(reg) => *reg
                    }.map(|x| f32::from_bits(x));
                    let dot_product = |a: [f32; 4], b: [f32; 4]| {
                        a[0] * b[0] +
                        a[1] * b[1] +
                        a[2] * b[2] +
                        a[3] * b[3]
                    };
                    let value = [
                        dot_product(a0, x),
                        dot_product(a1, x),
                        dot_product(a2, x),
                        dot_product(a3, x),
                    ];
                    dest_reg[i] = value.map(|x| x.to_bits());
                }
            },
            _ => panic!("GPU: Unimplemented shader instruction: {:?}", instruction),
        }
        Some(())
    }

    fn read_scalar_register<'io>(&self, address: RegisterAddress<Scalar>, run_context: &mut ShadingUnitRunContext<'io>) -> RegisterRead<'static, u32> {
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

    fn read_vector_register<'io>(&self, address: RegisterAddress<Vector>, run_context: &mut ShadingUnitRunContext<'io>) -> RegisterRead<'static, [u32; 4]> { 
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

    fn write_vector_register<'io>(&mut self, address: RegisterAddress<Vector>, run_context: &mut ShadingUnitRunContext<'io>) -> Option<&'static mut [[u32; 4]; CORE_COUNT]> {
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

    fn write_scalar_register<'io>(&mut self, address: RegisterAddress<Scalar>, run_context: &mut ShadingUnitRunContext<'io>) -> Option<&'static mut [u32; CORE_COUNT]> {
        let register_ptr = {
            let register_ref = match address {
                RegisterAddress::Local(index, ..) => Some(&mut self.scalar_locals[index as usize]),
                RegisterAddress::Output(index, ..) => Some(&mut run_context.scalar_output_array[index as usize]),
                _ => None
            };
            register_ref.map(|r| (r as *mut _))
        };
        register_ptr.map(|r| unsafe { &mut *r })
    }
}

pub(crate) fn read_bytes_u8(bytes: &[u8], offset: usize) -> u8 {
    if offset >= bytes.len() {
        0
    } else {
        bytes[offset]
    }
}

pub(crate) fn read_bytes_u16(bytes: &[u8], offset: usize) -> u16 {
    if offset + 2 > bytes.len() {
        0
    } else {
        ((bytes[offset + 1] as u16) << 0) |
        ((bytes[offset + 0] as u16) << 8)
    }
}

pub(crate) fn read_bytes_u32(bytes: &[u8], offset: usize) -> u32 {
    if (offset + 4) > bytes.len() {
        0
    } else {
        ((bytes[offset + 0] as u32) <<  0) |
        ((bytes[offset + 1] as u32) <<  8) |
        ((bytes[offset + 2] as u32) << 16) |
        ((bytes[offset + 3] as u32) << 24)
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

pub fn f32_bits_to_inorm8_bits(bits: u32) -> u8 {
    (f32::from_bits(bits) * std::i8::MAX as f32) as i32 as i8 as u8
}

pub fn f32_bits_to_unorm8_bits(bits: u32) -> u8 {
    (f32::from_bits(bits) * std::u8::MAX as f32) as u32 as u8
}

pub fn f32_bits_to_inorm16_bits(bits: u32) -> u16 {
    (f32::from_bits(bits) * std::i16::MAX as f32) as i32 as i16 as u16
}

pub fn f32_bits_to_unorm16_bits(bits: u32) -> u16 {
    (f32::from_bits(bits) * std::u16::MAX as f32) as u32 as u16
}

pub fn f32_bits_to_inorm32_bits(bits: u32) -> u32 {
    (f32::from_bits(bits) * std::i32::MAX as f32) as i32 as u32
}

pub fn f32_bits_to_unorm32_bits(bits: u32) -> u32 {
    (f32::from_bits(bits) * std::u32::MAX as f32) as u32
}

