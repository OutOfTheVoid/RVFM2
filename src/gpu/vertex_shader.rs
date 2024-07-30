use super::{buffer::BufferModule, shader::*, texture::TextureModule};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VertexInputAssignment {
    pub input: u8,
    pub src_buffer: u8,
    pub offset: u32,
    pub stride: u32,
    pub t: ShaderInputType,
    pub c: ShaderCardinality,
}

#[derive(Debug)]
pub(crate) struct VertexState {
    pub inputs: Vec<VertexInputAssignment>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VertexShaderResult {
    pub remaining_count: usize,
    pub remaining_offset: usize,
}

pub const VERTEX_SCALAR_INPUT_BUILTIN_VERTEX_ID        : usize = 0x00;
pub const VERTEX_SCALAR_INPUT_BUILTIN_PROVOKING_VERTEX : usize = 0x01;
pub const VERTEX_SCALAR_INPUT_USER_OFFSET              : usize = 0x10;

pub const VERTEX_SCALAR_OUTPUT_BUILTIN_VERTEX_DISCARD  : usize = 0x00;
pub const VERTEX_SCALAR_OUTPUT_USER_OFFSET             : usize = 0x10;

pub const VERTEX_VECTOR_OUTPUT_BUILTIN_VERTEX_POSITION : usize = 0x00;
pub const VERTEX_VECTOR_OUTPUT_USER_OFFSET             : usize = 0x10;

pub struct VertexShaderCall<'a> {
    pub shader        : &'a [ShaderInstruction],
    pub state         : &'a VertexState,

    pub vertex_count  : usize,
    pub vertex_offset : usize,

    pub shading_unit_context     : &'a mut ShadingUnitContext,
    pub shading_unit_run_context : ShadingUnitRunContext<'a>,
    
    pub buffer_modules  : &'a mut [BufferModule; 256],
    pub texture_modules : &'a mut [TextureModule; 64],
}

pub fn run_vertex_shader(mut call: VertexShaderCall<'_>) -> VertexShaderResult {
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
        let element_size = match input_assignment.t {
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
        match input_assignment.c {
            ShaderCardinality::Scalar => {
                (0..invocation_count).for_each(|i| {
                    call.shading_unit_run_context.scalar_input_array[input_assignment.input as usize][i] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i);
                });
            },
            ShaderCardinality::V2 => {
                (0..invocation_count).for_each(|i| {
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][0] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + 0);
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][1] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + element_size);
                });
            },
            ShaderCardinality::V3 => {
                (0..invocation_count).for_each(|i| {
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][0] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + element_size * 0);
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][1] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + element_size * 1);
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][2] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + element_size * 2);
                });
            },
            ShaderCardinality::V4 => {
                (0..invocation_count).for_each(|i| {
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][0] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + element_size * 0);
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][1] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + element_size * 1);
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][2] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + element_size * 2);
                    call.shading_unit_run_context.vector_input_array[input_assignment.input as usize][i][3] = read_fn(call.buffer_modules[input_assignment.src_buffer as usize].bytes(), input_assignment.offset as usize + input_assignment.stride as usize * i + element_size * 3);
                });
            }
        }
    }

    let mut instructions = call.shader.iter();
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
