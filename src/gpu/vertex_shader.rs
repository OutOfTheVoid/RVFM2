use super::{buffer::BufferModule, shader::{read_bytes_u16, read_bytes_u32, read_bytes_u8, ShaderInstruction, ShadingUnitContext, ShadingUnitRunContext, CORE_COUNT}, texture::TextureModule};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VertexInputCardinality {
    Scalar,
    V2,
    V3,
    V4
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VertexInputType {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VertexInputAssignment {
    input: u8,
    src_buffer: u8,
    offset: u32,
    stride: u32,
    t: VertexInputType,
    c: VertexInputCardinality,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Interpolation {
    ProvokingVertexFlat,
    Linear,
    Smooth,
    Max,
    Min,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VertexVaryingType {
    F32   (Interpolation),
    F32x2 (Interpolation),
    F32x3 (Interpolation),
    F32x4 (Interpolation),
    I32   (Interpolation),
    I32x2 (Interpolation),
    I32x3 (Interpolation),
    I32x4 (Interpolation),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VertexVaryingAssignment {
    pub vertex_output: u8,
    pub t: VertexVaryingType,
    pub scalar: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VertexConstantAssignment {
    pub vertex_constant: u8,
    pub source_buffer: u8,
    pub offset: u32,
    t: VertexInputType,
    c: VertexInputCardinality,
}

#[derive(Debug)]
pub(crate) struct VertexState {
    pub inputs: Vec<VertexInputAssignment>,
    pub varyings: Vec<VertexVaryingAssignment>,
    pub constants: Vec<VertexConstantAssignment>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VertexShaderResult {
    pub remaining_count: usize,
    pub remaining_offset: usize,
}

const VERTEX_SCALAR_INPUT_BUILTIN_VERTEX_ID        : usize = 0x00;
const VERTEX_SCALAR_INPUT_BUILTIN_PROVOKING_VERTEX : usize = 0x01;
const VERTEX_SCALAR_INPUT_USER_OFFSET              : usize = 0x10;

const VERTEX_SCALAR_OUTPUT_BUILTIN_VERTEX_DISCARD  : usize = 0x00;
const VERTEX_SCALAR_OUTPUT_USER_OFFSET             : usize = 0x10;

const VERTEX_VECTOR_OUTPUT_BUILTIN_VERTEX_POSITION : usize = 0x00;
const VERTEX_VECTOR_OUTPUT_USER_OFFSET             : usize = 0x10;

pub struct VertexShaderCall<'a> {
    state         : &'a VertexState,

    vertex_count  : usize,
    vertex_offset : usize,

    shading_unit_context     : &'a mut ShadingUnitContext,
    shading_unit_run_context : &'a mut ShadingUnitRunContext<'a>,
    
    buffer_modules  : &'a mut [BufferModule; 256],
    texture_modules : &'a mut [TextureModule; 64],
}

pub fn run_vertex_shader(call: &mut VertexShaderCall<'_>, instructions: &[ShaderInstruction]) -> VertexShaderResult {
    let invocation_count = call.vertex_count.min(CORE_COUNT);

    for v in 0..invocation_count {
        let builtin_v_id = v + call.vertex_offset;
        let builtin_provoking = if builtin_v_id % 3 == 0 { 1 } else { 0 };
        call.shading_unit_run_context.scalar_input_array[VERTEX_SCALAR_INPUT_BUILTIN_VERTEX_ID        as usize][v] = builtin_v_id as u32;
        call.shading_unit_run_context.scalar_input_array[VERTEX_SCALAR_INPUT_BUILTIN_PROVOKING_VERTEX as usize][v] = builtin_provoking;
    }
    call.shading_unit_run_context.scalar_output_array[VERTEX_SCALAR_OUTPUT_BUILTIN_VERTEX_DISCARD ][0..invocation_count].fill(0);
    call.shading_unit_run_context.vector_output_array[VERTEX_VECTOR_OUTPUT_BUILTIN_VERTEX_POSITION][0..invocation_count].fill([0; 4]);

    for input_assignment in call.state.inputs.iter() {
        let read_fn = match input_assignment.t {
            VertexInputType::UIntFromU8     => |bytes: &[u8], offset: usize|  read_bytes_u8 (bytes, offset) as u32                                         ,
            VertexInputType::UIntFromU16    => |bytes: &[u8], offset: usize|  read_bytes_u16(bytes, offset) as u32                                         ,
            VertexInputType::F32FromF32  |
            VertexInputType::UIntFromU32 |
            VertexInputType::IntFromI32     => |bytes: &[u8], offset: usize|  read_bytes_u32(bytes, offset),
            VertexInputType::IntFromI8      => |bytes: &[u8], offset: usize|  read_bytes_u8 (bytes, offset) as i8  as i32 as u32                           ,
            VertexInputType::IntFromI16     => |bytes: &[u8], offset: usize|  read_bytes_u16(bytes, offset) as i16 as i32 as u32                           ,
            VertexInputType::F32FromI8      => |bytes: &[u8], offset: usize| (read_bytes_u8 (bytes, offset) as i8  as f32                       ).to_bits(),
            VertexInputType::F32FromI16     => |bytes: &[u8], offset: usize| (read_bytes_u16(bytes, offset) as i16 as f32                       ).to_bits(),
            VertexInputType::F32FromI32     => |bytes: &[u8], offset: usize| (read_bytes_u32(bytes, offset) as i32 as f32                       ).to_bits(),
            VertexInputType::F32FromU8      => |bytes: &[u8], offset: usize| (read_bytes_u8 (bytes, offset)        as f32                       ).to_bits(),
            VertexInputType::F32FromU16     => |bytes: &[u8], offset: usize| (read_bytes_u16(bytes, offset)        as f32                       ).to_bits(),
            VertexInputType::F32FromU32     => |bytes: &[u8], offset: usize| (read_bytes_u32(bytes, offset)        as f32                       ).to_bits(),
            VertexInputType::F32FromUNorm8  => |bytes: &[u8], offset: usize| (read_bytes_u8 (bytes, offset)        as f32 / std::u8::MAX  as f32).to_bits(),
            VertexInputType::F32FromUNorm16 => |bytes: &[u8], offset: usize| (read_bytes_u16(bytes, offset)        as f32 / std::u16::MAX as f32).to_bits(),
            VertexInputType::F32FromUNorm32 => |bytes: &[u8], offset: usize| (read_bytes_u32(bytes, offset)        as f32                       ).to_bits(),
            VertexInputType::F32FromINorm8  => |bytes: &[u8], offset: usize| (read_bytes_u8 (bytes, offset) as i8  as f32 / std::i8::MAX  as f32).to_bits(),
            VertexInputType::F32FromINorm16 => |bytes: &[u8], offset: usize| (read_bytes_u16(bytes, offset) as i16 as f32 / std::i16::MAX as f32).to_bits(),
            VertexInputType::F32FromINorm32 => |bytes: &[u8], offset: usize| (read_bytes_u32(bytes, offset) as i32 as f32 / std::i32::MAX as f32).to_bits(),
        };
        let element_size = match input_assignment.t {
            VertexInputType::F32FromF32 |
            VertexInputType::F32FromI32 |
            VertexInputType::F32FromU32 |
            VertexInputType::F32FromINorm32 |
            VertexInputType::F32FromUNorm32 |
            VertexInputType::IntFromI32 |
            VertexInputType::UIntFromU32 => 4,
            VertexInputType::F32FromI16 |
            VertexInputType::F32FromU16 |
            VertexInputType::F32FromINorm16 |
            VertexInputType::F32FromUNorm16 |
            VertexInputType::IntFromI16 |
            VertexInputType::UIntFromU16 => 2,
            VertexInputType::F32FromI8 |
            VertexInputType::F32FromU8 |
            VertexInputType::F32FromUNorm8 |
            VertexInputType::F32FromINorm8 |
            VertexInputType::IntFromI8 |
            VertexInputType::UIntFromU8 => 1
        };
        match input_assignment.c {
            VertexInputCardinality::Scalar => {
                (0..invocation_count).for_each(|i| {
                    call.shading_unit_run_context.scalar_input_array[input_assignment.input as usize][i] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i);
                });
            },
            VertexInputCardinality::V2 => {
                (0..invocation_count).for_each(|i| {
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][0] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + 0);
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][1] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + element_size);
                });
            },
            VertexInputCardinality::V3 => {
                (0..invocation_count).for_each(|i| {
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][0] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + element_size * 0);
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][1] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + element_size * 1);
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][2] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + element_size * 2);
                });
            },
            VertexInputCardinality::V4 => {
                (0..invocation_count).for_each(|i| {
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][0] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + element_size * 0);
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][1] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + element_size * 1);
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][2] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + element_size * 2);
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][3] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + element_size * 3);
                });
            }
        }
    }

    for constant_assignment in call.state.constants.iter() {
        let bytes = call.buffer_modules[constant_assignment.source_buffer as usize].bytes();
        let read_fn = match constant_assignment.t {
            VertexInputType::UIntFromU8     => |bytes: &[u8], offset: usize|  read_bytes_u8 (bytes, offset) as u32                                         ,
            VertexInputType::UIntFromU16    => |bytes: &[u8], offset: usize|  read_bytes_u16(bytes, offset) as u32                                         ,
            VertexInputType::F32FromF32  |
            VertexInputType::UIntFromU32 |
            VertexInputType::IntFromI32     => |bytes: &[u8], offset: usize|  read_bytes_u32(bytes, offset),
            VertexInputType::IntFromI8      => |bytes: &[u8], offset: usize|  read_bytes_u8 (bytes, offset) as i8  as i32 as u32                           ,
            VertexInputType::IntFromI16     => |bytes: &[u8], offset: usize|  read_bytes_u16(bytes, offset) as i16 as i32 as u32                           ,
            VertexInputType::F32FromI8      => |bytes: &[u8], offset: usize| (read_bytes_u8 (bytes, offset) as i8  as f32                       ).to_bits(),
            VertexInputType::F32FromI16     => |bytes: &[u8], offset: usize| (read_bytes_u16(bytes, offset) as i16 as f32                       ).to_bits(),
            VertexInputType::F32FromI32     => |bytes: &[u8], offset: usize| (read_bytes_u32(bytes, offset) as i32 as f32                       ).to_bits(),
            VertexInputType::F32FromU8      => |bytes: &[u8], offset: usize| (read_bytes_u8 (bytes, offset)        as f32                       ).to_bits(),
            VertexInputType::F32FromU16     => |bytes: &[u8], offset: usize| (read_bytes_u16(bytes, offset)        as f32                       ).to_bits(),
            VertexInputType::F32FromU32     => |bytes: &[u8], offset: usize| (read_bytes_u32(bytes, offset)        as f32                       ).to_bits(),
            VertexInputType::F32FromUNorm8  => |bytes: &[u8], offset: usize| (read_bytes_u8 (bytes, offset)        as f32 / std::u8::MAX  as f32).to_bits(),
            VertexInputType::F32FromUNorm16 => |bytes: &[u8], offset: usize| (read_bytes_u16(bytes, offset)        as f32 / std::u16::MAX as f32).to_bits(),
            VertexInputType::F32FromUNorm32 => |bytes: &[u8], offset: usize| (read_bytes_u32(bytes, offset)        as f32                       ).to_bits(),
            VertexInputType::F32FromINorm8  => |bytes: &[u8], offset: usize| (read_bytes_u8 (bytes, offset) as i8  as f32 / std::i8::MAX  as f32).to_bits(),
            VertexInputType::F32FromINorm16 => |bytes: &[u8], offset: usize| (read_bytes_u16(bytes, offset) as i16 as f32 / std::i16::MAX as f32).to_bits(),
            VertexInputType::F32FromINorm32 => |bytes: &[u8], offset: usize| (read_bytes_u32(bytes, offset) as i32 as f32 / std::i32::MAX as f32).to_bits(),
        };
        let element_size = match constant_assignment.t {
            VertexInputType::F32FromF32 |
            VertexInputType::F32FromI32 |
            VertexInputType::F32FromU32 |
            VertexInputType::F32FromINorm32 |
            VertexInputType::F32FromUNorm32 |
            VertexInputType::IntFromI32 |
            VertexInputType::UIntFromU32 => 4,
            VertexInputType::F32FromI16 |
            VertexInputType::F32FromU16 |
            VertexInputType::F32FromINorm16 |
            VertexInputType::F32FromUNorm16 |
            VertexInputType::IntFromI16 |
            VertexInputType::UIntFromU16 => 2,
            VertexInputType::F32FromI8 |
            VertexInputType::F32FromU8 |
            VertexInputType::F32FromUNorm8 |
            VertexInputType::F32FromINorm8 |
            VertexInputType::IntFromI8 |
            VertexInputType::UIntFromU8 => 1
        };
        let value = match constant_assignment.c {
            VertexInputCardinality::Scalar => {
                call.shading_unit_run_context.scalar_constant_array[constant_assignment.vertex_constant as usize] = read_fn(bytes, constant_assignment.offset as usize);
            },
            VertexInputCardinality::V2 => {
                call.shading_unit_run_context.vector_constant_array[constant_assignment.vertex_constant as usize][0] = read_fn(bytes, constant_assignment.offset as usize + element_size * 0);
                call.shading_unit_run_context.vector_constant_array[constant_assignment.vertex_constant as usize][1] = read_fn(bytes, constant_assignment.offset as usize + element_size * 1);
            },
            VertexInputCardinality::V3 => {
                call.shading_unit_run_context.vector_constant_array[constant_assignment.vertex_constant as usize][0] = read_fn(bytes, constant_assignment.offset as usize + element_size * 0);
                call.shading_unit_run_context.vector_constant_array[constant_assignment.vertex_constant as usize][1] = read_fn(bytes, constant_assignment.offset as usize + element_size * 1);
                call.shading_unit_run_context.vector_constant_array[constant_assignment.vertex_constant as usize][2] = read_fn(bytes, constant_assignment.offset as usize + element_size * 2);
            },
            VertexInputCardinality::V4 => {
                call.shading_unit_run_context.vector_constant_array[constant_assignment.vertex_constant as usize][0] = read_fn(bytes, constant_assignment.offset as usize + element_size * 0);
                call.shading_unit_run_context.vector_constant_array[constant_assignment.vertex_constant as usize][1] = read_fn(bytes, constant_assignment.offset as usize + element_size * 1);
                call.shading_unit_run_context.vector_constant_array[constant_assignment.vertex_constant as usize][2] = read_fn(bytes, constant_assignment.offset as usize + element_size * 2);
                call.shading_unit_run_context.vector_constant_array[constant_assignment.vertex_constant as usize][3] = read_fn(bytes, constant_assignment.offset as usize + element_size * 3);
            },
        };
    }

    let mut instructions = instructions.iter();
    let mut continue_instructions = true;
    while let (Some(instruction), true) = (instructions.next(), continue_instructions) {
        if call.shading_unit_context.run_instruction(invocation_count, instruction, &mut call.shading_unit_run_context, &mut call.buffer_modules, &mut call.texture_modules).is_none() {
            continue_instructions = false;
        }
    }

    VertexShaderResult {
        remaining_count  : call.vertex_count  - invocation_count,
        remaining_offset : call.vertex_offset + invocation_count,
    }
}
